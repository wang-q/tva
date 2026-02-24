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

