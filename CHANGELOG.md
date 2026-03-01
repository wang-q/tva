# Change Log

## Unreleased - ReleaseDate

### Added

#### Data Manipulation
- **`blank`**: New command to replace consecutive identical values in selected columns (e.g., `1,2,3,3,4` -> `1,2,3,,4`). Supports custom replacements and case-insensitive comparison.

### Changed

#### Documentation
- **`CLAUDE.md` Update**: Updated source code organization section to accurately reflect the current project structure, particularly the `src/libs/` module hierarchy.

#### Testing
- **Test Infrastructure**: Introduced `tests/common/mod.rs` with `TvaCmd` struct to standardize command execution and assertion in tests.
- **Integration Tests Refactoring**: Migrated numerous integration tests (`cli_*.rs`) to use a unified `TvaCmd` helper in `tests/common/mod.rs`. This improves test readability, consistency, and maintenance.

## 0.2.0 - 2026-02-28

### Added

#### Data Import & Export
- **`from xlsx`**: Added support for importing Excel files (`.xlsx`).
- **`to csv`**: Added command to convert TSV to CSV.
- **`to xlsx`**: Added command to export TSV to Excel (`.xlsx`).

#### Performance & Core
- **Zero-Copy Architecture**: Migrated core commands (`select`, `filter`, `stats`, `sample`) to use a new zero-copy `TsvReader` and `ByteRecord` implementation, significantly reducing memory allocation.
- **SIMD Parsing**: Introduced `memchr`-based field splitting for parsing speeds approaching 1GB/s.
- **Benchmarks**: Added comprehensive benchmark suite (`benches/parse_benchmark.rs`) comparing `tva` against `csv`, `simd-csv`, and `tsv-utils`.

### Changed

#### Code Quality & CI
- **CI/CD**: Added GitHub Actions for documentation deployment (`docs.yml`).
- **Refactoring**: Optimized `append` and `bin` commands to use the new `TsvReader`.

## 0.1.0 - 2026-02-25

### Added

#### Core & Infrastructure
- **Initial Release**: High-performance, streaming-first TSV processing engine written in Rust.
- **Unified Field Selection**: Support for selecting columns by index (`1,2`), range (`1-5`), name (`header_name`), and wildcard (`*_id`).
- **Streaming I/O**: Efficient processing of large files with constant memory usage (O(1)) for most commands.
- **Transparent Compression**: Automatic handling of `.gz` input files.

#### Data Manipulation
- **`select`**: Select, reorder, and exclude columns using rich syntax.
- **`slice`**: Select rows by index or range (e.g., `1-10`, `100-`).
- **`filter`**: Filter rows using numeric, string, regex, and date predicates.
- **`sample`**: Random sampling support (Bernoulli, Reservoir, Weighted) and shuffling.
- **`sort`**: External sort implementation for handling large datasets that exceed memory.
- **`reverse`**: Reverse the order of lines (like `tac`).
- **`transpose`**: Swap rows and columns (matrix transposition).

#### Statistics & Aggregation
- **`stats`**: Compute summary statistics (sum, mean, median, mode, var, stddev) with grouping support.
- **`bin`**: Discretize numeric values into bins for histograms.
- **`uniq`**: Deduplicate rows with support for counting and specific field selection.

#### Reshaping (Tidyr-style)
- **`longer`**: Pivot wide data to long format (comparable to `pivot_longer`).
- **`wider`**: Pivot long data to wide format (comparable to `pivot_wider`).

#### Combining & Splitting
- **`join`**: Relational joins (inner, left, outer, anti) on specified keys.
- **`append`**: Concatenate TSV files with header awareness and source tracking.
- **`split`**: Split files by line count, random bucket, or key value.

#### Utilities
- **`check`**: Validate TSV structure and ensure consistent field counts.
- **`md`**: Convert TSV data to Markdown tables for documentation.
- **`from-csv`**: Import CSV files and convert them to TSV.
- **`nl`**: Add line numbers to rows.
- **`keep-header`**: Execute shell commands on data rows while preserving the header.

## 0.0.1 - 2026-02-21

Project skeleton
