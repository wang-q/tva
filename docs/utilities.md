# Formatting & Utilities Documentation

* **`check`**: Validate TSV file structure.
* **`nl`**: Add line numbers.
* **`keep-header`**: Run a shell command on the body, preserving the header.

## `check`

Checks TSV file structure for consistent field counts.

### Usage

```bash
tva check [files...]
```

It validates that every line in the file has the same number of fields as the first line. If a
mismatch is found, it reports the error line and exits with a non-zero status.

### Examples

Check a single file:

```bash
tva check docs/data/household.tsv
```

Output:

```
2 lines, 5 fields
```

## `nl`

Adds line numbers to TSV rows.

### Usage

```bash
tva nl [files...] [options]
```

Options:

* `-H` / `--header`: Treat the first line as a header. The header line is not numbered, and a "line"
  column is added to the header.
* `-s <STR>` / `--header-string <STR>`: Set the header name for the line number column (implies
  `-H`).
* `-n <N>` / `--start-number <N>`: Start numbering from N (default: 1).

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

The first line of the first input file is printed immediately. The remaining lines (and all lines
from subsequent files) are piped to the specified command. The output of the command is then
printed.

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
