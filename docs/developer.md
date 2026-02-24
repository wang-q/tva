# Developer Guide

## Help text style guide

* **`about`**: Third-person singular, describing the TSV operation
  (e.g., "Converts TSV to markdown table", "Deduplicates TSV rows").
* **`after_help`**: Use raw string `r###"..."###`.
    * **Description**: Short paragraph of what the subcommand does and its trade-offs.
    * **Notes**: Bullet points starting with `*`.
        * TSV input: `* Supports plain text and gzipped (.gz) TSV files`
        * Stdin behavior:
            * Single-input tools (e.g. `md`): `* Reads from stdin if input file is 'stdin' or no input file is given`
            * Multi-input tools (e.g. `uniq`): `* Reads from stdin if no input file is given or if input file is 'stdin'`
        * Memory-heavy tools (e.g. `uniq`): `* Keeps a hash for each unique row; does not count occurrences`
    * **Examples**: Numbered list (`1.`, `2.`) with code blocks indented by 3 spaces.
* **Arguments**:
    * **Input**: `infile` (single) or `infiles` (multiple).
        * Help (single): `Input TSV file to process (default: stdin).`
        * Help (multiple): `Input TSV file(s) to process`.
    * **Output**: `outfile` (`-o`, `--outfile`).
        * Help: `Output filename. [stdout] for screen`.
* **Terminology**:
    * Prefer "TSV" when referring to files.
    *   Use "row" / "column" in help text where it makes sense.

## Testing Strategy

We reuse the extensive test suite from upstream `tsv-utils` to ensure behavioral compatibility.

1.  **Golden Tests**:
    *   Copy input files and expected output files from `tsv-utils/tests/` to `tva/tests/data/<tool>/`.
2.  **CLI Tests**:
    *   Use `assert_cmd` in `tests/cli_<tool>.rs` to run `tva` commands.
    *   Compare stdout against the upstream golden output.
3.  **Behavior Alignment**:
    *   `tva` aims to produce identical output to `tsv-utils` for supported flags.
    *   Intentional deviations (e.g., error message format) are documented and tested separately.

## Architecture & Modules

### Infrastructure (`src/libs/`)

*   **Field Parsing (`libs::fields`)**:
    *   Implements the unified field syntax.
    *   Numeric intervals: `1,3-5`.
    *   Header-aware selection: `user_id`, wildcards (`*_time`), ranges (`start_col-end_col`), and escape sequences.
    *   Used by `select`, `join`, `uniq`, `stats`, `sample`, etc.
*   **Input Abstraction (`libs::io`)**:
    *   `reader`: Handles `stdin`, files, and `.gz` decompression transparently.
    *   `InputSource`: Provides a unified view for iterating over multiple input files.
*   **Number Formatting & Statistics (`libs::number`)**:
    *   Ensures consistent printing of floating-point numbers across tools.
    *   Implements R-compatible quantile calculations (`stats`).
*   **CLI Parsing**:
    *   Uses `clap` for argument parsing.
    *   Enforces consistent flag naming and help text styles (as per the guide above).

### Subcommand Implementation Status

*   **`md`**: Converts TSV to Markdown. Replaces `tsv-pretty` for display purposes.
*   **`append`**: Concatenates files with header awareness. Supports source tracking (`--track-source`).
*   **`uniq`**: Deduplicates rows. Supports field selection and equivalence classes.
*   **`nl`**: Adds line numbers.
*   **`keep-header`**: Runs shell commands on data rows while preserving headers.
*   **`check`**: Validates TSV structure (field counts). Fail-fast on jagged rows.
*   **`transpose`**: Swaps rows and columns. Strict matrix requirement.
*   **`sort`**: External sort implementation for large TSV files. Supports key selection.
*   **`from-csv`**: Imports CSV to TSV using Rust's `csv` crate.
*   **`select`**: Column selection, reordering, and exclusion. Supports rich header syntax.
*   **`sample`**: Random sampling (Bernoulli, Reservoir, Weighted) and shuffling.
*   **`split`**: Splits files by line count, random bucket, or key.
*   **`join`**: Inner/Left/Outer/Anti joins on specified keys.
*   **`filter`**: Row filtering with numeric, string, regex, and date predicates.
*   **`stats`**: Summary statistics (sum, mean, median, mode, etc.) with grouping.
*   **`longer`**: Reshapes wide to long (pivot_longer). Supports column selection by index/name/wildcard. Features: custom names for key/value columns (`--names-to`, `--values-to`), prefix stripping (`--names-prefix`), complex name parsing (`--names-sep`, `--names-pattern`), and NA dropping (`--values-drop-na`). Requires a header row.
*   **`wider`**: Reshapes long to wide (pivot_wider). Supports single column for names and values. Features: explicit ID columns (`--id-cols`), missing value filling (`--values-fill`), and column name sorting (`--names-sort`).
*   **`reverse`**: Reverses the order of lines (like `tac`), with optional header preservation.
*   **`bin`**: Discretizes numeric values into bins (for histograms). Supports custom width and alignment.

### Planned Features (Inspired by Datamash & R)

*   **Extended Statistics**:
    *   Add `q1` (25%), `q3` (75%), `iqr`, `skewness`, `kurtosis` to `stats`.
*   **Fill Missing Values**:
    *   `fill`: Implements forward/backward fill and constant value fill for missing data.

### Design: `slice`

*   **Goal**: 灵活地切片/分页（按行号选择）和删除行，统一 `head`/`tail`/`sed` 的功能。
*   **Usage**: `tva slice [options] [infiles]`
*   **Options**:
    *   `--rows` (`-r`): 指定要操作的行范围。支持多次指定。
    *   `--invert` (`-v`): 反转选择逻辑。
        *   不加 `-v` (Keep Mode): 仅保留 `-r` 选中的行（Whitelist）。
        *   加 `-v` (Drop Mode): 删除 `-r` 选中的行（Blacklist）。
    *   `--header` (`-H`): 始终保留第一行作为表头输出。
        *   **Keep Mode**: 即使第一行不在 `-r` 范围内，也会强制输出。
        *   **Drop Mode**: 即使第一行在 `-r` 范围内（如 `-r 1-5`），也会强制保留（不被删除）。
*   **Range Syntax**:
    *   `N`: 单行 (e.g., `5`)
    *   `N-M`: 闭区间 (e.g., `5-10`)
    *   `N-`: 从 N 开始到结束 (e.g., `5-`)
    *   `-M`: 从开始到 M (e.g., `-5`, 等同于 `1-5`)
*   **Examples**:
    *   **Keep (Slicing)**:
        *   `tva slice -r 10-20 file.tsv`: 仅输出第 10 到 20 行。
        *   `tva slice -r 1-5 -r 10-15 file.tsv`: 输出第 1-5 行和第 10-15 行。
    *   **Drop (Excluding)**:
        *   `tva slice -r 5 -v file.tsv`: 删除第 5 行。
        *   `tva slice -r 1-5 -v file.tsv`: 删除前 5 行。
    *   **Header Preservation**:
        *   `tva slice -H -r 100-110 file.tsv`: 输出表头 + 第 100 到 110 行（分页预览）。
        *   `tva slice -H -r 1-10 -v file.tsv`: 删除前 10 行数据，但保留表头（即删除第 2-10 行，保留第 1 行）。
