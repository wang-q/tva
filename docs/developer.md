# Developer Guide

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

### Subcommand Implementation Status

<details>
<summary>Implemented Commands</summary>

`append`, `bin`, `check`, `filter`, `from-csv`, `join`, `keep-header`, `longer`, `md`, `nl`, `reverse`, `sample`, `select`, `slice`, `sort`, `split`, `stats`, `transpose`, `uniq`, `wider`

</details>

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
*   **dplyr Core Patterns**:
    *   **Safe Joins**:
        *   **Concept**: Prevent accidental Cartesian explosions in `join`.
        *   **Action**: Add `--relationship` flag (e.g., `one-to-one`, `many-to-one`) to validate keys during join. Default to warning or error on unexpected many-to-many matches.
    *   **Tidy Selection DSL**:
        *   **Concept**: Decoupled, expressive column selection logic.
        *   **Action**: Enhance `src/libs/fields.rs` to support regex (`matches('^date_')`), predicates (`where(is_numeric)`), and set operations (`-colA`), usable across `select`, `wider`, `longer`.
    *   **Window Functions**:
        *   **Concept**: Context-aware row operations (rank, lead, lag).
        *   **Action**: Implement sliding window logic for `filter` and `stats` (e.g., `filter --expr "val > mean(val)"` within groups).
    *   **Torture Testing**:
        *   **Concept**: Robustness against malformed/edge-case data.
        *   **Action**: Create `tests/torture/` for fuzzing inputs (empty files, jagged rows, massive columns) to ensure zero panics.

### Documentation Plan (Inspired by tsv-utils)

*   **Reference Structure**:
    *   Create `docs/tool_reference.md` as a central index linking to individual tool documentation, similar to `tsv-utils/docs/ToolReference.md`.
    *   Create `docs/common_options.md` to document shared flags (Header handling, Field syntax, Input/Output buffering), reducing redundancy in individual help files.
*   **Performance**:
    *   Create `docs/performance.md`: Placeholder for benchmarks against `tsv-utils`, `datamash`, and `qsv`.

### Implementation Details

To help users get started quickly, we aim to provide dedicated documentation files for related groups of commands, similar to `docs/reshape.md`.

<details>
<summary>Completed Documentation</summary>

*   [Reshape](reshape.md) (`longer`, `wider`)
*   [Filtering](filtering.md) (`filter`)
*   [Selection](selection.md) (`select`, `slice`, `sample`)
*   [Design](design.md) (Architecture, Comparison)
*   [Statistics](statistics.md) (`stats`, `bin`, `uniq`)
*   [Ordering](ordering.md) (`sort`, `reverse`, `transpose`)
*   [Combining](combining.md) (`join`, `append`, `split`)
*   [Utilities](utilities.md) (`check`, `md`, `from-csv`, `nl`, `keep-header`)

</details>
*   **`docs/performance.md`** (Planned):
    *   Benchmarks: vs `tsv-utils`, `xsv`, `awk`.
    *   Methodology: Dataset descriptions and test cases.
