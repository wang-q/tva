# tva Common Conventions

This document defines the naming and behavior conventions for parameters shared across tva
subcommands to ensure a consistent user experience.

## Header Handling

Headers are the column name rows in data files. Different commands have different header processing
requirements, but parameter naming should remain consistent.

**Quick Selection:**

* Need column names for field references? Use `--header` (standard TSV) or `--header-hash1` (TSV
  with comments).
* Just skip header lines? Use `--header-lines N` (first N lines) or `--header-hash` (comment lines
  only).

**Header Detection Modes** (mutually exclusive):

* **Modes that provide column names** (`header_args_with_columns()`):

    * `--header` / `-H`: **FirstLine** mode
        - Takes the first line as column names.
        - Simplest mode for standard TSV files.
        - `lines` is empty, `column_names_line` is the first line.

    * `--header-hash1`: **HashLines1** mode
        - Takes consecutive `#` lines plus the next line as header.
        - **Graceful degradation**: If no `#` lines exist, uses the first line as column names (
          behaves like `--header`).
        - `lines` contains only `#` lines (empty if no `#` lines); column names line is stored
          separately.

  Commands using these modes: `append`, `bin`, `blank`, `fill`, `filter`, `join`, `longer`, `nl`,
  `reverse`, `select`, `stats`, `uniq`, `wider`.

* **Modes that don't provide column names** (`header_args()`):

    * `--header-lines N`: **LinesN** mode
        - Takes up to N lines as header (fewer if file is shorter).
        - Does not extract column names.
        - `lines` contains up to n lines, `column_names_line` is None.

    * `--header-hash`: **HashLines** mode
        - Takes all consecutive `#` lines as header (metadata only).
        - No column names line is extracted.
        - `lines` contains `#` lines, `column_names_line` is None.

  Commands using these modes: `check`, `slice`, `sort`.

**Library Implementation:**

* Use `TsvReader::read_header_mode(mode)` to read headers.
* Returns `HeaderInfo { lines, column_names_line }` where:
    - `lines`: all header lines read from input
    - `column_names_line`: the line containing column names (None if mode doesn't provide column
      names)
* Mode behavior:
    - `FirstLine`: `lines` is empty, `column_names_line` is the first line
    - `LinesN(n)`: `lines` contains up to n lines read, `column_names_line` is None
    - `HashLines`: `lines` contains all consecutive `#` lines, `column_names_line` is None
    - `HashLines1`: `lines` contains only `#` lines (empty if no `#` lines), `column_names_line` is
      the column names line

**Special Commands:**

* `split`: Uses `--header-in-out` (input has header, output writes header, default) or
  `--header-in-only` (input has header, output does not write header). `--header` is an alias for
  `--header-in-out`.
* `keep-header`: Uses `--lines N` / `-n` to specify number of header lines (default: 1)
* `sample`: Uses simple `--header` / `-H` flag (treats first non-empty line as header)
* `transpose`: Does not support header modes (processes all lines as data)

**Multi-file Header Behavior:**

* When using multiple input files with header mode enabled, the header from the first file is read
  and written to output.
* Headers from subsequent files are skipped.

## Input/Output Conventions

### Parameter Naming

| Type                | Parameter Name     | Description                            |
|---------------------|--------------------|----------------------------------------|
| Single file input   | `infile`           | Positional argument                    |
| Multiple file input | `infiles`          | Positional argument, supports multiple |
| Output file         | `--outfile` / `-o` | Optional, defaults to stdout           |

### Special Values

- `stdin` or `-`: Read from standard input
- `stdout`: Output to standard output (used with `--outfile`)

## Field Selection Syntax

Commands that support field selection (e.g., `select`, `filter`, `sort`) use a unified field syntax.

* **1-based Indexing**
    - Fields are numbered starting from 1 (following Unix `cut`/`awk` convention).
    - Example: `1,3,5` selects the 1st, 3rd, and 5th columns.

* **Field Names**
    - Requires the `--header` flag (or command-specific header option).
    - Names are case-sensitive.
    - Example: `date,user_id` selects columns named "date" and "user_id".

* **Ranges**
    - Numeric Ranges: `start-end`. Example: `2-4` selects columns 2, 3, and 4.
    - Name Ranges: `start_col-end_col`. Selects all columns from `start_col` to
      `end_col` inclusive, based on their order in the header.
    - Reverse Ranges: `5-3` is automatically treated as `3-5`.

* **Wildcards**
    - `*` matches any sequence of characters in a field name.
    - Example: `user_*` selects `user_id`, `user_name`, etc.
    - Example: `*_time` selects `start_time`, `end_time`.

* **Escaping**
    - Special characters in field names (like space, comma, colon, dash, star)
      must be escaped with `\`.
    - Example: `Order\ ID` selects the column "Order ID".
    - Example: `run\:id` selects "run:id".

* **Exclusion**
    - Negative selection is typically handled via a separate flag (e.g.,
      `--exclude` in `select`), but uses the same field syntax.

## Numeric Parameter Conventions

| Parameter           | Description        | Example           |
|---------------------|--------------------|-------------------|
| `--lines N` / `-n`  | Specify line count | `--lines 100`     |
| `--fields N` / `-f` | Specify fields     | `--fields 1,2,3`  |
| `--delimiter`       | Field delimiter    | `--delimiter ','` |

## Random and Sampling

| Parameter       | Description                             |
|-----------------|-----------------------------------------|
| `--seed N`      | Specify random seed for reproducibility |
| `--static-seed` | Use fixed default seed                  |

## Boolean Flags

Boolean flags use `--flag` to enable, without a value:

- `--header` not `--header true`
- `--append` / `-a` not `--append true`

## Error Handling

All commands follow the same error output format:

```
tva <command>: <error message>
```

Serious errors return non-zero exit codes.
