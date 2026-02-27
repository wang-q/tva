# tva Design

This document outlines the core design decisions behind `tva`, drawing inspiration from the original [TSV Utilities](https://github.com/eBay/tsv-utils) by eBay.

## Why TSV?

The Tab-Separated Values (TSV) format is chosen over Comma-Separated Values (CSV) for several key reasons, especially in data mining and large-scale data processing contexts:

### 1. No Escapes = Reliability & Speed
*   **CSV Complexity**: CSV uses escape characters (usually quotes) to handle delimiters (commas) and newlines within fields. Parsing this requires a state machine, which is slower and prone to errors in ad-hoc scripts.
*   **TSV Simplicity**: TSV disallows tabs and newlines within fields. This means:
    *   **Parsing is trivial**: `split('\t')` works reliably.
    *   **Record boundaries are clear**: Every newline is a record separator.
    *   **Performance**: Highly optimized routines can be used to find delimiters.
    *   **Robustness**: No "malformed CSV" errors due to incorrect quoting.

### 2. Unix Tool Compatibility
*   Traditional Unix tools (`cut`, `awk`, `sort`, `join`, `uniq`) work seamlessly with TSV files by specifying the delimiter (e.g., `cut -f1`).
*   **The CSV Problem**: Standard Unix tools fail on CSV files with quoted fields or newlines. This forces CSV toolkits (like `xsv`) to re-implement standard operations (sorting, joining) just to handle parsing correctly.
*   **The TSV Advantage**: `tva` leverages the simplicity of TSV. While `tva` provides its own `sort` and `join` for header awareness and Windows support, the underlying data remains compatible with the vast ecosystem of standard Unix text processing tools.

## Why Rust?

`tva` is implemented in Rust, differing from the original TSV Utilities (written in D).

### 1. Safety & Performance
*   **Memory Safety**: Rust's ownership model ensures memory safety without a garbage collector, crucial for high-performance data processing tools.
*   **Zero-Cost Abstractions**: High-level constructs (iterators, closures) compile down to efficient machine code, often matching or beating C/C++.
*   **Predictable Performance**: No GC pauses means consistent throughput for large datasets.

### 2. Cross-Platform & Deployment
*   **Single Binary**: Rust compiles to a static binary with no runtime dependencies (unlike Python or Java).
*   **Windows Support**: Rust has first-class support for Windows, making `tva` easily deployable on non-Unix environments (a key differentiator from many Unix-centric tools).

## Design Goals

### 1. Unix Philosophy
*   **Do One Thing Well**: Each subcommand (`filter`, `select`, `stats`) focuses on a specific task.
*   **Pipeable**: Tools read from stdin and write to stdout by default, enabling powerful pipelines:
    ```bash
    tva filter --gt score:0.9 data.tsv | tva select name,score | tva sort -k score
    ```
*   **Streaming**: Stateless where possible to support infinite streams and large files.

### 2. Header Awareness
*   Unlike generic Unix tools, `tva` is aware of headers.
*   **Field Selection**: Columns can be selected by name (`--fields user_id`) rather than just index.
*   **Header Preservation**: Operations like `filter` or `sample` automatically preserve the header row.

### 3. TSV-first
*   Default separator is TAB.
*   Processing revolves around the "Row + Field" model.
*   CSV is treated as an import format (`from-csv`), but core logic is TSV-centric.

### 4. Explicit CLI & Fail-fast
*   Options should be explicit (no "magic" behavior).
*   Strict error handling: mismatched field counts or broken headers result in immediate error exit (stderr + non-zero status), rather than silent truncation.

### 5. High Performance
*   Aim for single-pass processing.
*   Avoid unnecessary allocations and sorting.

## Implementation Details

`tva` adopts several optimization strategies similar to `tsv-utils` to ensure high performance:

### 1. Buffered I/O
*   **Input**: Uses `std::io::BufReader` to minimize system calls when reading large files. Transparently handles `.gz` files (via `flate2`).
*   **Output**: Uses `std::io::BufWriter` to batch writes, significantly improving throughput for commands that produce large output.

### 2. Zero-Copy & Re-use
*   **String Reuse**: Where possible, `tva` reuses allocated string buffers (e.g., via `read_line` into a cleared String) to avoid the overhead of repeated memory allocation and deallocation.
*   **Iterator-Based Processing**: Leverages Rust's iterator lazy evaluation to process data line-by-line without loading entire files into memory, enabling processing of datasets larger than RAM.

## Performance Architecture & Benchmarks

`tva` is built on a philosophy of "Zero-Copy" and "SIMD-First". We continuously benchmark different parsing strategies to ensure `tva` remains the fastest tool for TSV processing.

### Parsing Strategy Evolution

We compared 11 different parsing strategies to find the optimal balance between speed and correctness.

| Strategy | Throughput | Description |
| :--- | :--- | :--- |
| **Chunked Reader Sim** | **875 MiB/s** | **The Future**. Reads fixed-size chunks (e.g. 8KB) and processes with SIMD. Realistic high-performance model. |
| **Memchr2 SIMD Loop** | 865 MiB/s | **The Core**. Uses SIMD to find both `\t` and `\n` simultaneously. Faster than `csv` crate because it ignores quotes. |
| **Memchr Reused Buffer** | 806 MiB/s | Uses SIMD but processes line-by-line. Slightly slower due to function call overhead. |
| **csv crate** | 687 MiB/s | The standard Rust CSV parser. Highly optimized state machine, but pays a cost for handling quotes and escapes. |
| **Manual Byte Loop** | 519 MiB/s | Scalar loop (no SIMD). Proves that SIMD provides a ~60% speedup. |
| **Std Split Iterator** | 420 MiB/s | Standard `line.split('\t')`. Good, but not great. |
| **Naive Split** | 160 MiB/s | `line.split().collect()`. Allocating a `Vec` for every line is a performance killer. Avoid this! |

### Key Insights

1.  **I/O is the Bottleneck**: The standard `BufReader::lines()` allocates a new `String` for every line, capping throughput at ~400 MiB/s regardless of parsing speed. To break this barrier, `tva` must use chunked reading and zero-copy slicing.
2.  **TSV > CSV**: By strictly enforcing the TSV standard (no quotes, no escapes), we can use simpler, faster SIMD instructions (`memchr2`) that outperform even the most optimized CSV parsers (`simd-csv` or `csv` crate).
3.  **Zero Allocation**: The fastest parser is the one that allocates nothing. `tva` strives to reuse buffers and yield slices (`&[u8]`) instead of allocating new strings.

### Why is `simd-csv` still faster?

Despite our optimizations, `simd-csv` (1.15 GiB/s) still outperforms our `Chunked Reader` (875 MiB/s) by ~30%. Why?

1.  **Instruction-Level Parallelism (ILP)**:
    *   `memchr2` processes data in sequence: load -> find -> branch -> process -> repeat.
    *   `simd-csv` uses hand-written AVX2 intrinsics that can process larger blocks (32+ bytes) and speculatively execute multiple checks in parallel, minimizing branch mispredictions.
2.  **L1 Cache Efficiency**:
    *   `simd-csv`'s hybrid state machine is extremely compact.
    *   Our chunked reader introduces some overhead for buffer management (`copy_within`) and boundary checks.
3.  **The "Good Enough" Threshold**:
    *   Reaching 875 MiB/s means we can parse a 1GB file in ~1.2 seconds. This is already faster than most I/O subsystems (NVMe SSDs typically sustain 2-3 GB/s, but file system overhead matters).
    *   Further optimization requires dropping down to `unsafe` hand-written assembly, which trades safety and maintainability for diminishing returns.

## Common Behavior & Syntax

`tva` tools share a consistent set of behaviors and syntax conventions, making them easy to learn and combine.

### Field Syntax

All tools use a unified syntax to identify fields (columns). See [Field Syntax Documentation](help/fields.md) for details.

*   **Index**: `1` (first column), `2` (second column).
*   **Range**: `1-3` (columns 1, 2, 3).
*   **List**: `1,3,5`.
*   **Name**: `user_id` (requires `--header`).
*   **Wildcard**: `user_*` (matches `user_id`, `user_name`, etc.).
*   **Exclusion**: `--exclude 1,2` (select all except 1 and 2).

### Header Processing

*   **Input**: Most tools accept a `--header` (or `-H`) flag to indicate the first line of input is a header. This enables field selection by name.
    *   Note: The `longer` and `wider` commands assume a header by default.
*   **Output**: When `--header` is used, `tva` ensures the header is preserved in the output (unless explicitly suppressed).
*   **No Header**: Without this flag, the first row is treated as data. Field selection is limited to indices (no names).
*   **Multiple Files**: If processing multiple files with `--header`:
    *   The header from the **first** file is written to output.
    *   Headers from subsequent files are **skipped** (assumed to be identical to the first).
    *   **Validation**: Field counts must be consistent; `tva` fails immediately on jagged rows.

### Multiple Files & Standard Input

*   **Standard Input**: If no files are provided, or if `-` is used as a filename, `tva` reads from standard input (stdin).
*   **Concatenation**: When multiple files are provided, `tva` processes them sequentially as a single continuous stream of data (logical concatenation).
    *   Example: `tva filter --gt value:10 file1.tsv file2.tsv` processes both files.

## Comparison with Other Tools

`tva` is designed to coexist with and complement other excellent open-source tools for tabular data. It combines the strict, high-performance nature of `tsv-utils` with the cross-platform accessibility and modern ecosystem of Rust.

| Feature | `tva` (Rust) | `tsv-utils` (D) | `xsv` / `qsv` (Rust) | `datamash` (C) |
| :--- | :--- | :--- | :--- | :--- |
| **Primary Format** | TSV (Strict) | TSV (Strict) | CSV (Flexible) | TSV (Default) |
| **Escapes** | No | No | Yes | No |
| **Header Aware** | Yes | Yes | Yes | Partial |
| **Field Syntax** | Names & Indices | Names & Indices | Names & Indices | Indices |
| **Platform** | Cross-platform | Unix-focused | Cross-platform | Unix-focused |
| **Performance** | High | High | High (CSV cost) | High |

### Detailed Breakdown

*   **[tsv-utils](https://github.com/eBay/tsv-utils-d)** (D):
    *   The direct inspiration for `tva`. `tva` aims to be a Rust-based alternative that is easier to install (no D compiler needed) and extends functionality (e.g., `sample`, `slice`).

*   **[xsv](https://github.com/BurntSushi/xsv) / [qsv](https://github.com/jqnatividad/qsv)** (Rust):
    *   The premier tools for **CSV** processing.
    *   Because they must handle CSV escapes, they are inherently more complex than TSV-only tools.
    *   Use these if you must work with CSVs directly; use `tva` if you can convert to TSV for faster, simpler processing.

*   **[GNU Datamash](https://www.gnu.org/software/datamash/)** (C):
    *   Excellent for statistical operations (groupby, pivot) on TSV files.
    *   `tva stats` is similar but adds header awareness and named field selection, making it friendlier for interactive use.

*   **[Miller (mlr)](https://github.com/johnkerl/miller)** (C):
    *   A powerful "awk for CSV/TSV/JSON". Supports many formats and complex transformations.
    *   Miller is a DSL (Domain Specific Language); `tva` follows the "do one thing well" Unix philosophy with separate subcommands.

*   **[csvkit](https://github.com/wireservice/csvkit)** (Python):
    *   Very feature-rich but slower due to Python overhead. Great for converting obscure formats (XLSX, DBF) to CSV/TSV.

*   **[GNU shuf](https://www.gnu.org/software/coreutils/)** (C):
    *   Standard tool for random permutations.
    *   `tva sample` adds specific data science sampling methods: weighted sampling (by column value) and Bernoulli sampling.

