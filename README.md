# tva

Tab-separated Values Assistant.
Fast, reliable TSV processing toolkit in Rust.

[![Build](https://github.com/wang-q/tva/actions/workflows/build.yml/badge.svg)](https://github.com/wang-q/tva/actions)
[![codecov](https://codecov.io/gh/wang-q/tva/graph/badge.svg?token=c76YHsQnW7)](https://codecov.io/gh/wang-q/tva)
[![license](https://img.shields.io/github/license/wang-q/tva)](https://github.com/wang-q/tva)

## Overview

`tva` is a high-performance command-line toolkit for processing tabular data (especially TSV files). It is inspired by [eBay's tsv-utils](https://github.com/eBay/tsv-utils) and built in Rust, designed for speed, safety, and modern tooling.

It is designed for:
- Large files that are too big for Excel or Pandas but too small for distributed systems (Hadoop/Spark).
- Command-line data pipelines (Unix philosophy).
- Data exploration, cleaning, and transformation.

## Features & Design

`tva` follows the **Unix philosophy** and is built for **speed** and **reliability**:

*   **Streaming & Unix-like**: Reads from stdin, writes to stdout, stateless, and pipe-friendly.
*   **TSV-first**: Optimized for tab-separated data; no complex CSV escaping.
*   **Header Aware**: Select fields by name (`--fields user_id`) or index, preserving headers automatically.
*   **Fail-fast**: Strict error handling for data integrity (no silent truncation).
*   **High Performance**: Single-pass processing with minimal allocation.

See [Design Documentation](docs/design.md) for details.

## Commands

### Selection & Sampling
See [Selection & Sampling Documentation](docs/selection.md).

- **`select`**: Select, reorder, and rename columns.
- **`slice`**: Slice rows by index (keep or drop). Supports multiple ranges and header preservation.
- **`sample`**: Randomly sample rows (Bernoulli, reservoir, weighted).

### Filtering
See [Row Filtering Documentation](docs/filtering.md).

- **`filter`**: Filter rows based on numeric, string, regex, or date criteria.

### Statistics & Summary
- **`stats`**: Calculate summary statistics (sum, mean, median, min, max, etc.) with grouping.
- **`bin`**: Discretize numeric values into bins (useful for histograms).
- **`uniq`**: Deduplicate rows or count unique occurrences (supports equivalence classes).

### Reshaping
See [Reshaping Documentation](docs/reshape.md).

- **`longer`**: Reshape wide to long (unpivot). Requires a header row.
- **`wider`**: Reshape long to wide (pivot). Supports aggregation via `--op` (sum, count, etc.).

**Comparison: `stats` vs `wider`**

| Feature | `stats` (Group By) | `wider` (Pivot) |
| :--- | :--- | :--- |
| **Goal** | Summarize to rows | Reshape to columns |
| **Output** | Long / Tall | Wide / Matrix |

### Transformation & Combination
- **`join`**: Join two files based on common keys (inner, left, outer, anti).
- **`append`**: Concatenate multiple TSV files, handling headers correctly.
- **`split`**: Split a file into multiple files (by size, key, or random).
- **`sort`**: Sort TSV files (external sort for large files).
- **`transpose`**: Transpose rows and columns.
- **`reverse`**: Reverse the order of lines (like `tac`), with optional header preservation.

### Formatting & Utilities
- **`check`**: Validate TSV file structure (column counts, encoding).
- **`from-csv`**: Convert CSV to TSV.
- **`md`**: Convert TSV to Markdown table for display.
- **`nl`**: Add line numbers to rows.
- **`keep-header`**: Run a shell command on the body of a TSV file, preserving the header.

## Common Options & Syntax

### Field Selection
Most commands support selecting fields using a common syntax:
- **Index**: `1` (first column), `2` (second column).
- **Range**: `1-3` (columns 1, 2, 3).
- **List**: `1,3,5`.
- **Name**: `user_id` (requires `--header`).
- **Wildcard**: `user_*` (matches `user_id`, `user_name`, etc.).
- **Exclusion**: `--exclude 1,2` (select all except 1 and 2).

### Header Handling
- **Flag**: Use `--header` or `-H` to indicate the input file has a header row. (Note: `longer` command assumes header by default).
- **Output**: The header row is propagated to the output (unless explicitly suppressed by a command).
- **Multi-File Behavior**: When processing multiple files with `--header`:
    - The first file defines the column names.
    - Headers in subsequent files are automatically skipped (assumed to match the first file).
    - **Validation**: Field counts must be consistent; `tva` fails immediately on jagged rows.
- **No Header**: Without this flag, the first row is treated as data. Field selection is limited to indices (no names).

## Installation

```bash
cargo install --path .
```

## Examples

```bash
tva md tests/genome/ctg.range.tsv --num -c 2
tva md tests/genome/ctg.range.tsv --fmt --digits 2

tva uniq tests/genome/ctg.tsv tests/genome/ctg.tsv
tva uniq tests/genome/ctg.tsv -f 2

tva nl tests/genome/ctg.tsv

```

## Author

Qiang Wang <wang-q@outlook.com>

## License

MIT.
Copyright by Qiang Wang.
