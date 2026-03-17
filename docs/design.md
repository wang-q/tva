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

### Latest Benchmark Results

Data sample: `1\tJohn\tDoe\t30\tNew York\n...` (3 rows repeated 1000 times)

| Implementation | Time | Throughput | Notes |
| :--- | :--- | :--- | :--- |
| **simd-csv** | **67 µs** | **1.05 GiB/s** | Hybrid SIMD state machine, performance ceiling |
| **Tva TsvReader** | **72 µs** | **1.00 GiB/s** | **Current**: Zero-copy Reader + SIMD (memchr) |
| **Memchr Reused Buffer** | 82 µs | 878 MiB/s | Line-by-line memchr, limited by function call overhead |
| **csv crate** | 111 µs | 652 MiB/s | Classic DFA state machine, correctness baseline |
| **Naive Split** | 443 µs | 163 MiB/s | Original implementation, slowest |

**Key Findings**:

1. **Performance Leap**: Our new `Tva TsvReader` (1.00 GiB/s) is **2.6x faster** than the old
   version, reaching **95%** of `simd-csv` performance.
2. **Bottleneck Shift**: The bottleneck has moved from "memory allocation/IO" to "field iteration".

### Key Insights

1. **I/O is the Bottleneck**: `BufReader::lines()` allocates a new `String` per line, capping
   throughput at ~400 MiB/s. Chunked reading with zero-copy slicing breaks this barrier.
2. **TSV > CSV**: By enforcing no quotes/escapes, we use simpler SIMD (`memchr2`) that outperforms
   optimized CSV parsers.
3. **Zero Allocation**: The fastest parser allocates nothing. `tva` reuses buffers and yields
   slices (`&[u8]`).

### Why is `simd-csv` still faster?

`simd-csv` (1.15 GiB/s) outperforms our `Chunked Reader` (875 MiB/s) by ~30% due to:

1. **Instruction-Level Parallelism**: Hand-written AVX2 intrinsics process 32+ byte blocks with
   speculative execution.
2. **L1 Cache Efficiency**: `simd-csv`'s hybrid state machine is extremely compact.
3. **The "Good Enough" Threshold**: At 875 MiB/s, we parse 1GB in ~1.2 seconds—already faster than
   most I/O subsystems. Further optimization requires `unsafe` assembly with diminishing returns.

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

### Design Goals

The goal is to build a TSV parser that is faster than `rust-csv` and lighter than `simd-csv`:

1. **Pure SIMD Scanning**: Without quote handling, we can use SIMD (`memchr` or `std::simd`) to
   blindly scan for `\t` and `\n`. No backtracking, no state maintenance.
2. **Absolute Zero-Copy**: CSV parsers must copy memory when encountering escaped quotes (
   `Cow<str>`). TSV parsers can always return slices (`&str` or `&[u8]`) from the original buffer,
   completely avoiding allocation.
3. **Line-Level Parallelism**: Since `\n` has unique semantics in TSV (record separator), we can
   safely split large files into chunks, align at `\n` boundaries, and parse in parallel. This is
   difficult in CSV where `\n` might be inside quotes.

### Can We Do Better?

**TSV's Advantage**: TSV has **no quotes**. This means we don't need to check if a `memchr` hit is
inside quotes or maintain a `Quoted` state like `simd-csv` does.

**Simpler SIMD**: We only need to find `\t` and `\n`. This is fewer special characters than CSV's
3-4, reducing register pressure.

**Theoretical Limit**: If `simd-csv` can achieve 1.12 GiB/s while handling quote logic, a pure TSV
parser should theoretically reach memory bandwidth limits (or at least 2-3 GiB/s).

**Action Items**:
We don't need complex hybrid state machines. We just need an optimized `memchr2(b'\t', b'\n')` loop
with efficient buffer management. Our `Memchr Reused Buffer` (806 MiB/s) already validates this
approach.

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
