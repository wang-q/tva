# tva Design

This document outlines the core design decisions behind `tva`, drawing inspiration from the
original [TSV Utilities](https://github.com/eBay/tsv-utils) by eBay.

## Why TSV?

The Tab-Separated Values (TSV) format is chosen over Comma-Separated Values (CSV) for several key
reasons, especially in data mining and large-scale data processing contexts:

### 1. No Escapes = Reliability & Speed

* **CSV Complexity**: CSV uses escape characters (usually quotes) to handle delimiters (commas) and
  newlines within fields. Parsing this requires a state machine, which is slower and prone to errors
  in ad-hoc scripts.
* **TSV Simplicity**: TSV disallows tabs and newlines within fields. This means:
    * **Parsing is trivial**: `split('\t')` works reliably.
    * **Record boundaries are clear**: Every newline is a record separator.
    * **Performance**: Highly optimized routines can be used to find delimiters.
    * **Robustness**: No "malformed CSV" errors due to incorrect quoting.

### 2. Unix Tool Compatibility

* Traditional Unix tools (`cut`, `awk`, `sort`, `join`, `uniq`) work seamlessly with TSV files by
  specifying the delimiter (e.g., `cut -f1`).
* **The CSV Problem**: Standard Unix tools fail on CSV files with quoted fields or newlines. This
  forces CSV toolkits (like `xsv`) to re-implement standard operations (sorting, joining) just to
  handle parsing correctly.
* **The TSV Advantage**: `tva` leverages the simplicity of TSV. While `tva` provides its own `sort`
  and `join` for header awareness and Windows support, the underlying data remains compatible with
  the vast ecosystem of standard Unix text processing tools.

## Why Rust?

`tva` is implemented in Rust, differing from the original TSV Utilities (written in D).

### 1. Safety & Performance

* **Memory Safety**: Rust's ownership model ensures memory safety without a garbage collector,
  crucial for high-performance data processing tools.
* **Zero-Cost Abstractions**: High-level constructs (iterators, closures) compile down to efficient
  machine code, often matching or beating C/C++.
* **Predictable Performance**: No GC pauses means consistent throughput for large datasets.

### 2. Cross-Platform & Deployment

* **Single Binary**: Rust compiles to a static binary with no runtime dependencies (unlike Python or
  Java).
* **Windows Support**: Rust has first-class support for Windows, making `tva` easily deployable on
  non-Unix environments (a key differentiator from many Unix-centric tools).

## Design Goals

### 1. Unix Philosophy

* **Do One Thing Well**: Each subcommand (`filter`, `select`, `stats`) focuses on a specific task.
* **Pipeable**: Tools read from stdin and write to stdout by default, enabling powerful pipelines:
  ```bash
  tva filter --gt score:0.9 data.tsv | tva select name,score | tva sort -k score
  ```
* **Streaming**: Stateless where possible to support infinite streams and large files.

### 2. Header Awareness

* Unlike generic Unix tools, `tva` is aware of headers.
* **Field Selection**: Columns can be selected by name (`--fields user_id`) rather than just index.
* **Header Preservation**: Operations like `filter` or `sample` automatically preserve the header
  row.

### 3. TSV-first

* Default separator is TAB.
* Processing revolves around the "Row + Field" model.
* CSV is treated as an import format (`from-csv`), but core logic is TSV-centric.

### 4. Explicit CLI & Fail-fast

* Options should be explicit (no "magic" behavior).
* Strict error handling: mismatched field counts or broken headers result in immediate error exit (
  stderr + non-zero status), rather than silent truncation.

### 5. High Performance

* Aim for single-pass processing.
* Avoid unnecessary allocations and sorting.

### 6. Single-Threaded by Default

**Core Philosophy: Single-threaded extreme performance + external parallel tools**

`tva` adopts a **single-threaded** model for most data processing scenarios. This is not a technical
limitation, but an active choice based on Unix philosophy:

1. **Do One Thing Well**: `tva` focuses on streaming data parsing, transformation, and statistics,
   leaving parallel scheduling complexity to specialized tools (like GNU Parallel).
2. **Don't Reinvent the Wheel**: GNU Parallel is already a mature, powerful parallel task scheduler.
   Rather than implementing complex thread pools and task distribution inside `tva`, it's better to
   make `tva` the best partner for Parallel.
3. **Determinism and Simplicity**: Single-threaded models make data processing order naturally
   deterministic, debugging easier, and greatly reduce memory management complexity and overhead (
   lock-free, zero-copy easier to achieve).

## Implementation Details

`tva` adopts several optimization strategies similar to `tsv-utils` to ensure high performance:

### 1. Buffered I/O

* **Input**: Uses `std::io::BufReader` to minimize system calls when reading large files.
  Transparently handles `.gz` files (via `flate2`).
* **Output**: Uses `std::io::BufWriter` to batch writes, significantly improving throughput for
  commands that produce large output.

### 2. Zero-Copy & Re-use

* **String Reuse**: Where possible, `tva` reuses allocated string buffers (e.g., via `read_line`
  into a cleared String) to avoid the overhead of repeated memory allocation and deallocation.
* **Iterator-Based Processing**: Leverages Rust's iterator lazy evaluation to process data
  line-by-line without loading entire files into memory, enabling processing of datasets larger than
  RAM.

## Performance Architecture & Benchmarks

`tva` is built on a philosophy of "Zero-Copy" and "SIMD-First". We continuously benchmark different
parsing strategies to ensure `tva` remains the fastest tool for TSV processing.

### Parsing Strategy Evolution

We compared multiple parsing strategies to find the optimal balance between speed and correctness.
The evolution shows a clear progression from naive implementations to hand-optimized SIMD:

1. **Naive Split** → **Memchr-based** → **Single-Pass SIMD** → **Hand-written SIMD**
2. Each step eliminates overhead: allocation, function calls, or redundant scanning.

### Latest Benchmark Results

#### Test Data 1: Short Fields, Few Columns (5 cols, ~8 bytes/field)

| Implementation | Time | Throughput | Notes |
| :--- | :--- | :--- | :--- |
| **TVA for_each_row (Single-Pass)** | **43 µs** | **1.63 GiB/s** | **Current**: Hand-written SIMD (SSE2), true single-pass |
| **simd-csv** | 80 µs | 905 MiB/s | Hybrid SIMD state machine, previous ceiling |
| **TVA for_each_line + memchr** | 87 µs | 830 MiB/s | Two-pass: SIMD for lines, memchr for fields |
| **Memchr Reused Buffer** | 113 µs | 639 MiB/s | Line-by-line memchr, limited by function call overhead |
| **csv crate** | 130 µs | 556 MiB/s | Classic DFA state machine, correctness baseline |
| **Naive Split** | 562 µs | 129 MiB/s | Original implementation, slowest |

#### Test Data 2: Wide Rows, Many Columns (20 cols, ~6 bytes/field)

| Implementation | Time | Throughput | Notes |
| :--- | :--- | :--- | :--- |
| **TVA for_each_row (Single-Pass)** | **128 µs** | **896 MiB/s** | **Current**: Hand-written SIMD (SSE2), true single-pass |
| **simd-csv** | 180 µs | 635 MiB/s | Hybrid SIMD state machine |
| **TVA for_each_line + memchr** | 247 µs | 463 MiB/s | Two-pass: SIMD for lines, memchr for fields |
| **Memchr Reused Buffer** | 344 µs | 333 MiB/s | Line-by-line memchr |
| **csv crate** | 320 µs | 358 MiB/s | Classic DFA state machine |
| **Naive Split** | 1167 µs | 98 MiB/s | Original implementation |

**Key Findings**:

1. **Performance Leap**: `for_each_row` achieves **1.63 GiB/s** on short fields—**1.8x faster** than
   `simd-csv` and **12.6x faster** than naive split. On wide rows, it maintains **896 MiB/s**,
   demonstrating consistent advantage across data shapes.
2. **Single-Pass Wins**: True single-pass scanning outperforms two-pass approaches by **~95%**
   regardless of row width, as more delimiter searches are eliminated.
3. **Scalability**: All implementations show expected throughput decrease on wide rows (more
   delimiters to process), but TVA's single-pass approach maintains the lead.

## TSV Parser Design

This section details the design of `tva`'s custom TSV parser, which leverages the simplicity of the
TSV format to achieve high performance.

### Format Differences: CSV vs TSV

| Feature | CSV (RFC 4180) | TSV (Simple) | Impact |
| :--- | :--- | :--- | :--- |
| **Delimiter** | `,` (variable) | `\t` (fixed) | TSV can hardcode delimiter, enabling SIMD optimization. |
| **Quotes** | Supports `"` wrapping | **Not supported** | TSV eliminates "in_quote" state machine, removing branch misprediction. |
| **Escapes** | `""` escapes quotes | None | TSV supports true zero-copy slicing without rewriting. |
| **Newlines** | Allowed in fields | **Not allowed** | TSV guarantees `\n` always means record end, enabling parallel chunking. |

### Implementation

**Architecture**:

```
src/libs/tsv/simd/
├── mod.rs    - DelimiterSearcher trait, platform abstraction
├── sse2.rs   - x86_64 SSE2 implementation (128-bit vectors)
└── neon.rs   - aarch64 NEON implementation (128-bit vectors)
```

**Key Design Decisions**:

1. **Hand-written SIMD**: Platform-specific searchers simultaneously scan for `\t` and `\n`,
   eliminating generic library overhead.

2. **Single-Pass Scanning**: All delimiter positions are found in one pass, storing field
   boundaries in a pre-allocated array. This eliminates the **~95%** overhead of two-pass approaches.

3. **Unified CR Handling**: Only `\t` and `\n` are searched during SIMD scan. When `\n` is found,
   we check if the preceding byte is `\r`. This reduces register pressure compared to searching
   for three characters simultaneously.

4. **Zero-Copy API**: `TsvRow` structs yield borrowed slices into the internal buffer,
   eliminating per-row allocation.

**Platform Support**:
- **x86_64**: SSE2 intrinsics (baseline for all x86_64 CPUs)
- **aarch64**: NEON intrinsics (baseline for all ARM64 CPUs)
- **Fallback**: `memchr2` for other platforms

### Performance Validation

| Metric | Target | Achieved | Status |
|:-------|:-------|:---------|:-------|
| Throughput (short fields) | 2-3 GiB/s | **1.63 GiB/s** | ✅ Near theoretical limit |
| Speedup vs `simd-csv` | 1.5-2x | **1.8x** | ✅ Exceeded target |
| Speedup vs memchr2 | 1.5-2x | **2.0x** | ✅ Achieved target |

**Key Insights**:

- **SSE2 over AVX2**: 128-bit SSE2 outperformed 256-bit AVX2. Wider registers added overhead
  without proportional gains for TSV's simple structure.
- **Single-Pass Architecture**: The dominant performance factor, providing **~95%** improvement
  over two-pass approaches regardless of data shape.

## Common Behavior & Syntax

`tva` tools share a consistent set of behaviors and syntax conventions, making them easy to learn
and combine.

### Field Syntax

All tools use a unified syntax to identify fields (columns).
See [Field Syntax Documentation](help/fields.md) for details.

* **Index**: `1` (first column), `2` (second column).
* **Range**: `1-3` (columns 1, 2, 3).
* **List**: `1,3,5`.
* **Name**: `user_id` (requires `--header`).
* **Wildcard**: `user_*` (matches `user_id`, `user_name`, etc.).
* **Exclusion**: `--exclude 1,2` (select all except 1 and 2).

### Header Processing

* **Input**: Most tools accept a `--header` (or `-H`) flag to indicate the first line of input is a
  header. This enables field selection by name.
    * Note: The `longer` and `wider` commands assume a header by default.
* **Output**: When `--header` is used, `tva` ensures the header is preserved in the output (unless
  explicitly suppressed).
* **No Header**: Without this flag, the first row is treated as data. Field selection is limited to
  indices (no names).
* **Multiple Files**: If processing multiple files with `--header`:
    * The header from the **first** file is written to output.
    * Headers from subsequent files are **skipped** (assumed to be identical to the first).
    * **Validation**: Field counts must be consistent; `tva` fails immediately on jagged rows.

### Multiple Files & Standard Input

* **Standard Input**: If no files are provided, or if `-` is used as a filename, `tva` reads from
  standard input (stdin).
* **Concatenation**: When multiple files are provided, `tva` processes them sequentially as a single
  continuous stream of data (logical concatenation).
    * Example: `tva filter --gt value:10 file1.tsv file2.tsv` processes both files.

## Comparison with Other Tools

`tva` is designed to coexist with and complement other excellent open-source tools for tabular data.
It combines the strict, high-performance nature of `tsv-utils` with the cross-platform accessibility
and modern ecosystem of Rust.

| Feature | `tva` (Rust) | `tsv-utils` (D) | `xsv` / `qsv` (Rust) | `datamash` (C) |
| :--- | :--- | :--- | :--- | :--- |
| **Primary Format** | TSV (Strict) | TSV (Strict) | CSV (Flexible) | TSV (Default) |
| **Escapes** | No | No | Yes | No |
| **Header Aware** | Yes | Yes | Yes | Partial |
| **Field Syntax** | Names & Indices | Names & Indices | Names & Indices | Indices |
| **Platform** | Cross-platform | Unix-focused | Cross-platform | Unix-focused |
| **Performance** | High | High | High (CSV cost) | High |

### Detailed Breakdown

* **[tsv-utils](https://github.com/eBay/tsv-utils-d)** (D):
    * The direct inspiration for `tva`. `tva` aims to be a Rust-based alternative that is easier to
      install (no D compiler needed) and extends functionality (e.g., `sample`, `slice`).

* **[xsv](https://github.com/BurntSushi/xsv) / [qsv](https://github.com/jqnatividad/qsv)** (Rust):
    * The premier tools for **CSV** processing.
    * Because they must handle CSV escapes, they are inherently more complex than TSV-only tools.
    * Use these if you must work with CSVs directly; use `tva` if you can convert to TSV for faster,
      simpler processing.

* **[GNU Datamash](https://www.gnu.org/software/datamash/)** (C):
    * Excellent for statistical operations (groupby, pivot) on TSV files.
    * `tva stats` is similar but adds header awareness and named field selection, making it
      friendlier for interactive use.

* **[Miller (mlr)](https://github.com/johnkerl/miller)** (C):
    * A powerful "awk for CSV/TSV/JSON". Supports many formats and complex transformations.
    * Miller is a DSL (Domain Specific Language); `tva` follows the "do one thing well" Unix
      philosophy with separate subcommands.

* **[csvkit](https://github.com/wireservice/csvkit)** (Python):
    * Very feature-rich but slower due to Python overhead. Great for converting obscure formats (
      XLSX, DBF) to CSV/TSV.

* **[GNU shuf](https://www.gnu.org/software/coreutils/)** (C):
    * Standard tool for random permutations.
    * `tva sample` adds specific data science sampling methods: weighted sampling (by column value)
      and Bernoulli sampling.

## Aggregation Architecture

This section provides a deep dive into the architectural differences between `tva` and other tools
like `xan` (Rust) and `tsv-utils` (D Language) in their aggregation module designs.

### tva: Runtime Polymorphism with SoA Memory Layout

**Design**: Hybrid Struct-of-Arrays (SoA). The Schema (`StatsProcessor`) builds the computation
graph at runtime, while the State (`Aggregator`) uses compact columnar storage (`Vec<f64>`,
`Vec<String>`). Computation logic is dynamically dispatched via `Box<dyn Calculator>` trait objects.

**Advantages**:

* **Memory Efficient**: Even with millions of groups, each group's `Aggregator` overhead is
  minimal (only a few `Vec` headers).
* **Modular**: Adding new operators only requires implementing the `Calculator` trait, completely
  decoupled from existing code.
* **Fast Compilation**: Compared to generic/template bloat, `dyn Trait` significantly reduces
  compile times and binary size.
* **Deterministic**: Uses `IndexMap` to guarantee that GroupBy output order matches the
  first-occurrence order in the input.

**Trade-offs**: Virtual function calls (`vtable`) have a tiny overhead compared to inlined code in
extremely high-frequency loops (e.g., 10 calls per row), but this is usually negligible in I/O-bound
CLI tools.

### Other Tools

**xan**: Uses enum dispatch (`enum Agg { Sum(SumState), ... }`) to avoid heap allocation, but
requires modifying core enum definitions to add new operators.

**tsv-utils (D)**: Uses compile-time template specialization for extreme performance, but has long
compile times and high code complexity.

**datamash (C)**: Uses sort-based grouping with O(1) memory, but requires pre-sorted input.

**dplyr (R)**: Uses vectorized mask evaluation, but depends on columnar storage and is unsuitable
for streaming.

## Expr Language

TVA's Expr language is designed for concise, shell-friendly data processing:

```
Source → Pest Parser → AST (Expr) → Direct Interpretation (eval)
              ↑______________________________↓
                    (Parse Cache)
```

### Design Principles

* **Conciseness**: Short syntax for common operations (e.g., `@1`, `@name` for column references).
* **Shell-friendly**: Avoids conflicts with Shell special characters (`$`, `` ` ``, `!`).
* **Streaming**: Row-by-row evaluation with no global state, suitable for big data.
* **Type-aware**: Recognizes numbers/dates when needed, treats data as strings by default for speed.
* **Error Handling**: Defaults to permissive mode (invalid operations return `null`).
* **Consistency**: Similar to jq/xan to reduce learning costs.

### Expr Engine Optimizations

| Optimization | Technique | Speedup |
|:-------------|:----------|:--------|
| Global Function Registry | `OnceLock` static registry | 35-57x |
| Parse Cache | `HashMap<String, Expr>` caching | 12x |
| Column Name Resolution | Compile-time name→index conversion | 3x |
| Constant Folding | Compile-time constant evaluation | 10x |
| HashMap (ahash) | Faster HashMap implementation | 6% |

**Details**:

* **Parse caching**: Expressions are parsed once and cached for all rows. Identical expressions
  reuse the cached AST.
* **Column name resolution**: When headers are available, `@name` references are resolved to
  `@index` at parse time for O(1) access.
* **Constant folding**: Constant sub-expressions (e.g., `2 + 3 * 4`) are pre-computed during
  parsing.
* **Function registry**: Built-in functions are looked up once and cached, avoiding repeated hash
  map lookups.
* **Hash algorithm**: Uses `ahash` for faster hash map operations.

For best performance, use column indices (`@1`, `@2`) instead of names.
