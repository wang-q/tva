# fill

Fills missing values in selected columns using the last non-missing value (down/LOCF) or a constant
value.

Behavior:

* Down (LOCF): By default, missing values are replaced with the most recent non-missing value in
  the same column.
* Constant: If `--value` / `-v` is provided, missing values are replaced with this constant
  string.
* Missing Definition: A value is considered "missing" if it matches the string provided by
  `--na` (default: empty string).
* Filling is stateful across file boundaries when multiple files are provided.

Input:

* Reads from files or standard input; multiple files are processed as one stream.
* Files ending in `.gz` are transparently decompressed.

Header behavior:

* Supports `--header` / `-H` and `--header-hash1` modes.
* When using header mode with multiple files, only the header from the first file is
  written; headers from subsequent files are skipped.

Field syntax:

* Use `-f` / `--field` to specify columns to fill.
* Columns can be specified by 1-based index or, if `-H` is used, by header name.
* Run `tva --help-fields` for a full description shared across tva commands.

Output:

* By default, output is written to standard output.
* Use `--outfile` to write to a file instead.

Examples:

1. Fill missing values in column 1 downwards
   `tva fill -H -f 1 data.tsv`

2. Fill missing values in columns 'category' and 'type' downwards
   `tva fill -H -f category -f type data.tsv`

3. Fill missing values in column 2 with "0"
   `tva fill -H -f 2 -v "0" data.tsv`

4. Treat "NA" as missing and fill downwards
   `tva fill -H -f 1 --na "NA" data.tsv`
