# nl

Adds line numbers to TSV rows. This is a simplified, TSV-aware version of the Unix
`nl` program with support for treating the first input line as a header.

Behavior:

* Prepends a line number column to each row of input data.
* Line numbers increase by 1 for each data line, continuously across all input files.
* Header lines are never numbered.
* Completely empty files are skipped and do not consume line numbers.
* Supports custom delimiters between the line number and line content.

Input:

* Reads from files or standard input.
* Files ending in `.gz` are transparently decompressed.
* When multiple files are given, lines are numbered continuously across files.
* Empty files (including files with only blank lines) are skipped.

Output:

* By default, output is written to standard output.
* Use `--outfile` / `-o` to write to a file instead.
* Each output line starts with the line number, followed by a delimiter, then the original line content.

Header behavior:

* `--header` / `-H`: Treats the first line of the input as a header. The header is
  written once at the top of the output with the line number column header prepended.
* `--header-string` / `-s`: Sets the header text for the line number column (default: "line").
  This option implies `--header`.
* When using header mode with multiple files, only the header from the first non-empty
  file is written; subsequent header lines are skipped and not numbered.

Numbering:

* `--start-number` / `-n`: The number to use for the first line (default: 1, can be negative).
* Numbers increase by 1 for each data line across all input files.

Examples:

1. Number lines of a TSV file
   `tva nl data.tsv`

2. Number lines with a header for the line number column
   `tva nl --header --header-string LINENUM data.tsv`

3. Number lines starting from 100
   `tva nl --start-number 100 data.tsv`

4. Number multiple files, preserving continuous line numbers
   `tva nl input1.tsv input2.tsv`

5. Read from stdin
   `cat input1.tsv | tva nl`

6. Use a custom delimiter
   `tva nl --delimiter ":" data.tsv`
