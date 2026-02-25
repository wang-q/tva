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
*   **`slice`**: Slice rows by index (keep or drop), supports range selection and header preservation.

### Planned Features (Inspired by Datamash, R, and qsv)

*   **Extended Statistics**:
    *   Add `q1` (25%), `q3` (75%), `iqr`, `skewness`, `kurtosis` to `stats`.
*   **Fill Missing Values**:
    *   `fill`: Implements forward/backward fill and constant value fill for missing data.
*   **Indexing Mechanism**:
    *   **Status**: `tva` is currently primarily stream-based.
    *   **Reference**: `qsv`'s core advantage is creating inverted indices (`.idx` files) for CSVs. This enables instant `slice`, `count`, and random access on GB-sized files.
    *   **Proposal**: Consider introducing an optional indexing mechanism for `tva`, particularly for large files requiring multiple passes.
*   **Apply Command (Complex Transformations)**:
    *   **Reference**: `qsv apply` supports column transformations based on string, date, math, and even NLP (fuzzy matching, sentiment analysis).
    *   **Proposal**: `tva`'s `select` currently leans towards selection. Consider enhancing its expression capabilities or adding an `apply` command to handle `datefmt` (date formatting) and `regex_replace`.
*   **Tidyr Parity (Advanced Reshaping)**:
    *   **Multi-measure Pivoting**:
        *   `longer`: Support `.value` sentinel in `--names-to` to pivot into multiple value columns simultaneously (e.g. `cols = c("x_1", "x_2", "y_1", "y_2")` -> `id, num, x, y`).
        *   `wider`: Allow `--values-from` to accept multiple columns, creating output columns like `val1_A`, `val1_B`, `val2_A`, `val2_B`.
    *   **Column Splitting/Merging**:
        *   `unpack`: Splits a single string column into multiple columns using a separator or regex (e.g., unpack "2023-10-27" into "year", "month", "day").
        *   `pack`: Combines multiple columns into a single string column using a template or separator (e.g., pack "Lat", "Lon" into "Coordinates").
    *   **Densification**:
        *   `complete`: Expose missing combinations of data factors (explicit missing rows).

### Documentation Plan (Inspired by tsv-utils)

*   **Reference Structure**:
    *   Create `docs/tool_reference.md` as a central index linking to individual tool documentation, similar to `tsv-utils/docs/ToolReference.md`.
    *   Create `docs/common_options.md` to document shared flags (Header handling, Field syntax, Input/Output buffering), reducing redundancy in individual help files.
*   **Performance**:
    *   Create `docs/performance.md`: Placeholder for benchmarks against `tsv-utils`, `datamash`, and `qsv`.

### Implementation Details

To help users get started quickly, we aim to provide dedicated documentation files for related groups of commands, similar to `docs/reshape.md`.

*   **`docs/reshape.md`** (Done):
    *   `longer`: Wide to long reshaping.
    *   `wider`: Long to wide reshaping (pivot).
*   **`docs/filtering.md`** (Done):
    *   `filter`: Row filtering syntax (numeric, string, regex, date).
*   **`docs/selection.md`** (Done):
    *   `select`: Column selection syntax (indices, names, wildcards, ranges).
    *   `slice`: Row selection by index/range.
    *   `sample`: Sampling methods (Bernoulli, Reservoir, Weighted).
*   **`docs/design.md`** (Done):
    *   Architecture: Why Rust? Why TSV?
    *   Design: No escapes, performance, Unix compatibility.
    *   Comparison: vs `tsv-utils`, `xsv`, `datamash`.
*   **`docs/statistics.md`** (Planned):
    *   `stats`: Summary statistics and grouping.
    *   `bin`: Discretization for histograms.
    *   `uniq`: Deduplication and counting.
*   **`docs/ordering.md`** (Done):
    *   `sort`: External sort for large files.
    *   `reverse`: Reverse the order of lines.
    *   `transpose`: Transpose rows and columns.
*   **`docs/combining.md`** (Planned):
    *   `join`: Join operations (Inner/Left/Outer/Anti).
    *   `append`: Concatenation with header awareness.
    *   `split`: Splitting files by count/key/random.
*   **`docs/formatting.md`** (Planned):
    *   `check`: Structure validation.
    *   `md`: Markdown conversion.
    *   `from-csv`: CSV import.
    *   `nl`, `keep-header`.
*   **`docs/performance.md`** (Planned):
    *   Benchmarks: vs `tsv-utils`, `xsv`, `awk`.
    *   Methodology: Dataset descriptions and test cases.
