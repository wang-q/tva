# Change Log

## Unreleased - ReleaseDate

## 0.3.0 - 2026-03-16

### Added

#### Expr Language (`expr` command)
- **New `expr` command**: Expression evaluation command for TSV data processing with support for:
    - Column references by name or index (`@NAME`, `@1`)
    - Whole row reference (`@0`)
    - Variables with persistence across rows (`@var`)
    - Pipeline operator (`|`) for chaining operations
    - Underscore placeholder (`_`) for piped values in multi-argument functions
    - Lambda expressions with `map`, `filter`, `reduce` functions
    - Method call syntax (`.upper()`, `.len()`)
    - List literals and operations
    - String, numeric, datetime, and hash functions
    - Type checking functions (`is_null`, `is_string`, `is_number`, etc.)
    - Meta functions for system info (`version`, `now`, `row`, etc.)
    - Short-circuit evaluation for logical operators (`and`, `or`)
    - String escape sequences and q-strings
    - Range function for number sequence generation
    - Sorting with `sort_by` and custom comparators
    - `--skip-null` flag to filter null results
    - Automatic header generation for expression output
- **Expression Engine Library** (`src/libs/expr/`):
    - **Parser** (`parser/`): Pest-based parser with modular builder pattern
        - `grammar.pest`: PEG grammar definition
        - `ast.rs`: AST node definitions with `Expr::Underscore` for placeholders
        - `builder/`: Modular parser components (binary, unary, primary, postfix, lambda, literal)
    - **Runtime** (`runtime/`): Expression evaluation engine
        - `mod.rs`: Core interpreter with `EvalContext` and `last_value` support
        - `value.rs`: Dynamic value type system with `Arc<str>` optimization
    - **Functions** (`functions/`): Comprehensive function library
        - `string.rs`: String manipulation (substr, replace, trim, etc.)
        - `numeric.rs`: Math functions (abs, round, sqrt, trig, etc.)
        - `datetime.rs`: Date/time parsing and formatting
        - `hash.rs`: Hash functions (md5, sha256, base64)
        - `list.rs`: List operations (map, filter, reduce, range, etc.)
        - `logical.rs`: Type checking and logical operations
        - `meta.rs`: System info functions
        - `regex.rs`: Regular expression support
        - `io.rs`: I/O functions (print, eprint)
        - `mod.rs`: Function registry with arity tracking
- **Two-stage compilation**: AST parsing with `ConcreteExpr` for optimized evaluation
- **Performance optimizations**:
    - Pre-resolved column names to indices
    - Parse caching for repeated expressions
    - Function call optimization
    - Replaced `std::collections::HashMap` with `ahash` for better performance

#### Documentation
- **Expr Language Documentation**: Comprehensive documentation for the expr language:
    - `docs/expr/design.md`: Design principles and syntax overview
    - `docs/expr/functions.md`: Complete function reference
    - `docs/expr/literals.md`: Literal types and syntax
    - `docs/expr/operators.md`: Operators and precedence
    - `docs/expr/rosetta.md`: Rosetta stone examples comparing with other tools

#### Testing
- **Comprehensive test coverage for expr engine**:
    - Unit tests for parser, AST, and runtime modules
    - Integration tests with real data files
    - Tests for all function modules (string, numeric, datetime, hash, list)
    - Edge case tests for numeric functions
    - Expression parsing and evaluation tests
    - Comprehensive documentation tests

### Changed

#### Refactoring
- **Parser modularization**: Restructured expression parser into modular builder components
- **Test consolidation**: Consolidated test files into module files for better organization
- **Documentation restructure**: Reorganized expression documentation into separate files

## 0.2.5 - 2026-03-12

### Added

#### New Commands
- **`header`**: New command for analyzing and displaying TSV file headers with support for all four header detection modes.

#### Plotting
- **`plot point`**: Terminal-based scatter plot visualization with support for:
    - Multiple Y columns and color grouping
    - Regression line fitting (Liang-Barsky clipping algorithm)
    - Path mode (`--path`) for connected line plots
- **`plot box`**: Box plot visualization with grouping support and outlier detection
- **`plot bin2d`**: 2D binning heatmap visualization with automatic binning and character density rendering

## 0.2.4 - 2026-03-10

### Added

#### Header Handling
- **Unified Header Detection Modes**: Introduced four header detection modes across multiple commands:
    - `--header` / `-H`: FirstLine mode - treats the first line as column names.
    - `--header-hash1`: HashLines1 mode - treats consecutive `#` lines plus the next line as header, with graceful fallback to FirstLine mode if no `#` lines exist.
    - `--header-lines N`: LinesN mode - treats up to N lines as header without extracting column names.
    - `--header-hash`: HashLines mode - treats all consecutive `#` lines as header metadata only.
- **Commands with full header mode support** (4 modes: `--header`, `--header-lines`, `--header-hash`, `--header-hash1`): `check`, `slice`, `sort`.
- **Commands with column-names header support** (2 modes: `--header`, `--header-hash1`): `append`, `bin`, `blank`, `fill`, `filter`, `join`, `longer`, `nl`, `reverse`, `select`, `stats`, `uniq`, `wider`.
- **New CLI utilities**: Added `header_args_with_columns()` and `header_args()` in `libs/cli.rs` for consistent header argument handling.
- **`tva --help-headers`**: Added global flag to display detailed header mode documentation.

#### Documentation
- **Common Conventions**: Added `docs/conventions.md` documenting shared parameter naming, header handling, I/O conventions, and field selection syntax across all commands.
- **Updated Help Text Style Guide**: Standardized help documentation format with consistent section ordering and formatting.

### Changed

#### Refactoring
- **Header handling**: Migrated all applicable commands to use the new `TsvReader::read_header_mode()` API for consistent header processing.

#### Testing
- **Expanded test coverage**: Added comprehensive tests for header modes across multiple commands.

## 0.2.3 - 2026-03-02

### Added

#### Data Import & Export
- **`from html`**: New command for extracting data from HTML files. Supports Query Mode (Pup-compatible), Table Mode (automatic extraction), and List Mode (structured row/column extraction).
- **`to md`**: Migrated from `tva md` to `tva to md`. Converts TSV files to Markdown tables with support for column alignment and numeric formatting.

#### Documentation
- **CSS Selectors Reference**: Added `docs/selectors.md` with a comprehensive guide for selectors used in `from html`.
- **Restructured Format Docs**: Split the monolithic format conversion guide into dedicated `docs/from.md` and `docs/to.md` for better clarity.

## 0.2.2 - 2026-03-01

### Added

#### Data Manipulation
- **`fill`**: New command to fill missing values in selected columns using various strategies (e.g., previous value, next value, specific constant).
- **`select`**: Added `--rest` flag to select all remaining columns not explicitly selected. Allowed combining `-f` (select) and `-e` (exclude) flags for flexible column selection.
- **`split`**: Enhanced file splitting with new strategies: by line count (`--lines`), by random bucket (`--bucket`), or by key field (`--key`).
- **`filter`**: Introduced a modular filter engine supporting complex field-based row filtering logic.

#### Statistics & Aggregation
- **`stats`**:
    - Added **Quantile** aggregation support.
    - Added **MAD** (Median Absolute Deviation) calculation.
    - Added `--delimiter` option to specify output delimiter.
    - Added `--exclude-missing` flag to ignore missing values in calculations.
    - Added support for custom output headers and missing value replacement.
    - Improved formatting options for numeric outputs.

### Changed

#### Performance
- **Numeric Parsing**: Switched to `lexical` crate for faster `f64` parsing in `stats` and other numeric-heavy commands.

#### Fixes
- **`stats`**: Corrected the calculation logic for Median Absolute Deviation (MAD).
- **`keep-header`**: Fixed handling of empty files or files containing only headers.
- **`nl`**: Fixed behavior when processing empty input files.
- **`append`**: Ensured input files are processed in the order specified on the command line; fixed line-buffered output behavior.

#### Refactoring
- **Modularization**: Extensive refactoring of `stats`, `sampling`, and `filter` modules to improve code organization and maintainability.

#### Documentation
- **Reshape Diagram**: Added a visual diagram to `docs/reshape.md` illustrating `longer`, `wider`, `fill`, and `blank` operations.

## 0.2.1 - 2026-03-01

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
