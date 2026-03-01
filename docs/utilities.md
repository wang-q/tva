# Formatting & Utilities Documentation

This document explains how to use the utility commands in `tva`: **`check`**, **`md`**, **`nl`**, and **`keep-header`**.

## `check`

Checks TSV file structure for consistent field counts.

### Usage

```bash
tva check [files...]
```

It validates that every line in the file has the same number of fields as the first line. If a mismatch is found, it reports the error line and exits with a non-zero status.

### Examples

Check a single file:
```bash
tva check docs/data/household.tsv
```
Output:
```
2 lines, 5 fields
```

## `md`

Converts a TSV file to a Markdown table.

### Usage

```bash
tva md [file] [options]
```

Options:
*   `--num`: Right-align numeric columns.
*   `--fmt`: Format numeric columns (thousands separators, fixed decimals) and implies `--num`.
*   `--digits <N>`: Set decimal precision for `--fmt` (default: 0).
*   `--center <cols>` / `--right <cols>`: Manually set alignment for specific columns.

### Examples

Basic markdown table:
```bash
tva md docs/data/household.tsv
```

Output:
```markdown
| family | dob_child1 | dob_child2 | name_child1 | name_child2 |
| ------ | ---------- | ---------- | ----------- | ----------- |
| 1      | 1998-11-26 | 2000-01-29 | J           | K           |
```

Format numbers with commas and 2 decimal places:
```bash
tva md docs/data/us_rent_income.tsv --fmt --digits 2
```

Output:
```markdown
| GEOID | NAME       | variable |  estimate |    moe |
| ----: | ---------- | -------- | --------: | -----: |
|  1.00 | Alabama    | income   | 24,476.00 | 136.00 |
|  1.00 | Alabama    | rent     |    747.00 |   3.00 |
|  2.00 | Alaska     | income   | 32,940.00 | 508.00 |
|  2.00 | Alaska     | rent     |  1,200.00 |  13.00 |
|  4.00 | Arizona    | income   | 27,517.00 | 148.00 |
|  4.00 | Arizona    | rent     |    972.00 |   4.00 |
|  5.00 | Arkansas   | income   | 23,789.00 | 165.00 |
|  5.00 | Arkansas   | rent     |    709.00 |   5.00 |
|  6.00 | California | income   | 29,454.00 | 109.00 |
|  6.00 | California | rent     |  1,358.00 |   3.00 |
```

## `nl`

Adds line numbers to TSV rows.

### Usage

```bash
tva nl [files...] [options]
```

Options:
*   `-H` / `--header`: Treat the first line as a header. The header line is not numbered, and a "line" column is added to the header.
*   `-s <STR>` / `--header-string <STR>`: Set the header name for the line number column (implies `-H`).
*   `-n <N>` / `--start-number <N>`: Start numbering from N (default: 1).

### Examples

Add line numbers (no header logic):
```bash
tva nl docs/data/household.tsv
```

Output:
```tsv
1	family	dob_child1	dob_child2	name_child1	name_child2
2	1	1998-11-26	2000-01-29	J	K
```

Add line numbers with header awareness:
```bash
tva nl -H docs/data/household.tsv
```

Output:
```tsv
line	family	dob_child1	dob_child2	name_child1	name_child2
1	1	1998-11-26	2000-01-29	J	K
```

## `keep-header`

Executes a shell command on the body of a TSV file, preserving the header.

### Usage

```bash
tva keep-header [files...] -- <command> [args...]
```

The first line of the first input file is printed immediately. The remaining lines (and all lines from subsequent files) are piped to the specified command. The output of the command is then printed.

### Examples

Sort a file while keeping the header at the top:
```bash
tva keep-header data.tsv -- sort
```

Grep for a pattern but keep the header:
```bash
tva keep-header docs/data/world_bank_pop.tsv -- grep "AFG"
```

Output:
```tsv
country	indicator	2000	2001
AFG	SP.URB.TOTL	4436311	4648139
AFG	SP.URB.GROW	3.91	4.66
```
