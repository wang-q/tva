# from csv

Converts CSV (Comma-Separated Values) input to TSV output.

Behavior:

* Parsing is delegated to the Rust `csv` crate, handling quoted fields, embedded
  delimiters, and newlines according to the CSV specification.
* TAB and newline characters found inside CSV fields are replaced with the
  strings specified by `--tab-replacement` and `--newline-replacement` (default: space).

Input:

* Reads from files or standard input.
* Files ending in `.gz` are transparently decompressed.
* Use `stdin` or omit the file argument to read from standard input.

Output:

* Each CSV record becomes one TSV line.
* Fields are joined with TAB characters.
* By default, output is written to standard output.
* Use `--outfile` to write to a file instead.

Examples:

1. Convert a CSV file to TSV
   `tva from csv data.csv > data.tsv`

2. Read CSV from stdin and convert to TSV
   `cat data.csv | tva from csv > data.tsv`

3. Use a custom delimiter (e.g., semicolon)
   `tva from csv --delimiter ';' data.csv`
