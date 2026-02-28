# tva

Tab-separated Values Assistant.
Fast, reliable TSV processing toolkit in Rust.

[![Build](https://github.com/wang-q/tva/actions/workflows/build.yml/badge.svg)](https://github.com/wang-q/tva/actions)
[![codecov](https://codecov.io/gh/wang-q/tva/graph/badge.svg?token=c76YHsQnW7)](https://codecov.io/gh/wang-q/tva)
[![license](https://img.shields.io/github/license/wang-q/tva)](https://github.com/wang-q/tva)
[![Documentation](https://img.shields.io/badge/docs-online-blue)](https://wang-q.github.io/tva/)

## Overview

`tva` (pronounced "Tee-Va") is a high-performance command-line toolkit written in **Rust** for processing tabular data. It brings the safety and speed of modern systems programming to the classic Unix philosophy.

**Inspiration**

*   [eBay's tsv-utils](https://github.com/eBay/tsv-utils) (discontinued): The primary reference for functionality and performance.
*   [GNU Datamash](https://www.gnu.org/software/datamash/): Statistical operations.
*   [R's tidyr](https://tidyr.tidyverse.org/): Reshaping concepts (`longer`, `wider`).

**Use Cases**

*   **"Middle Data"**: Files too large for Excel/Pandas but too small for distributed systems (Spark/Hadoop).
*   **Data Pipelines**: Robust CLI-based ETL steps compatible with `awk`, `sort`, etc.
*   **Exploration**: Fast summary statistics, sampling, and filtering on raw data.

**Design Principles**

*   **Single Binary**: A standalone executable with no dependencies, easy to deploy.
*   **Header Aware**: Manipulate columns by name or index.
*   **Fail-fast**: Strict validation ensures data integrity (no silent truncation).
*   **Streaming**: Stateless processing designed for infinite streams and large files.
*   **TSV-first**: Prioritizes the reliability and simplicity of tab-separated values.
*   **Performance**: Single-pass execution with minimal memory overhead.

See [Design Documentation](docs/design.md) for details.

**[Read the documentation online](https://wang-q.github.io/tva/)**

## Installation

Current release: 0.2.0

```bash
# Clone the repository and install via cargo
cargo install --force --path .
```

Or install the pre-compiled binary via the cross-platform package manager [cbp](https://github.com/wang-q/cbp) (supports older Linux systems with glibc 2.17+):


```bash
cbp install tva
```

You can also download the pre-compiled binaries from the [Releases](https://github.com/wang-q/tva/releases) page.

## Running Examples

The examples in the documentation use sample data located in the `docs/data/` directory. To run these examples yourself, we recommend cloning the repository:

```bash
git clone https://github.com/wang-q/tva.git
cd tva
```

Then you can run the commands exactly as shown in the docs (e.g., `tva select -f 1 docs/data/input.csv`).

Alternatively, you can download individual files from the [docs/data](https://github.com/wang-q/tva/tree/master/docs/data) directory on GitHub.

## Commands

### [Selection & Sampling](docs/selection.md)

- **`select`**: Select and reorder columns.
- **`slice`**: Slice rows by index (keep or drop). Supports multiple ranges and header preservation.
- **`sample`**: Randomly sample rows (Bernoulli, reservoir, weighted).

### [Filtering](docs/filtering.md)

- **`filter`**: Filter rows based on numeric, string, regex, or date criteria.

### [Ordering](docs/ordering.md)

- **`sort`**: Sorts rows based on one or more key fields.
- **`reverse`**: Reverses the order of lines (like `tac`), optionally keeping the header at the top.
- **`transpose`**: Swaps rows and columns (matrix transposition).

### [Statistics & Summary](docs/statistics.md)

- **`stats`**: Calculate summary statistics (sum, mean, median, min, max, etc.) with grouping.
- **`bin`**: Discretize numeric values into bins (useful for histograms).
- **`uniq`**: Deduplicate rows or count unique occurrences (supports equivalence classes).

### [Reshaping](docs/reshape.md)

- **`longer`**: Reshape wide to long (unpivot). Requires a header row.
- **`wider`**: Reshape long to wide (pivot). Supports aggregation via `--op` (sum, count, etc.).

**Comparison: `stats` vs `wider`**

| Feature | `stats` (Group By) | `wider` (Pivot) |
| :--- | :--- | :--- |
| **Goal** | Summarize to rows | Reshape to columns |
| **Output** | Long / Tall | Wide / Matrix |

### [Combining & Splitting](docs/combining.md)

- **`join`**: Join two files based on common keys (inner, left, outer, anti).
- **`append`**: Concatenate multiple TSV files, handling headers correctly.
- **`split`**: Split a file into multiple files (by size, key, or random).

### [Formatting & Utilities](docs/utilities.md)

- **`from`**: Convert other formats to TSV (csv, xlsx).
- **`to`**: Convert TSV to other formats (csv, xlsx).
- **`check`**: Validate TSV file structure (column counts, encoding).
- **`md`**: Convert TSV to Markdown table for display.
- **`nl`**: Add line numbers to rows.
- **`keep-header`**: Run a shell command on the body of a TSV file, preserving the header.

## Author

Qiang Wang <wang-q@outlook.com>

## License

MIT.
Copyright by Qiang Wang.
