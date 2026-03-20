# bin

Discretizes numeric values into bins. Useful for creating histograms or grouping
continuous data.

Behavior:

* Replaces the value in the target field with the bin start (lower bound).
* Formula: `floor((value - min) / width) * width + min`.
* Use `--new-name` to append as a new column instead of replacing.
* Commonly used with `stats --groupby` to compute statistics per bin.

Input:

* Reads from files or standard input; multiple files are processed as one stream.
* Files ending in `.gz` are transparently decompressed.

Header behavior:

* Supports `--header` / `-H` and `--header-hash1` modes.
* When using header mode with multiple files, only the header from the first file is
  written; headers from subsequent files are skipped.

Field syntax:

* The `--field` argument accepts a 1-based index or a header name (when using
  `--header`).
* Run `tva --help-fields` for a full description shared across tva commands.

Output:

* By default, output is written to standard output.
* Use `--outfile` to write to a file instead.

Examples:

1. Bin a numeric column with width 10
   `tva bin --width 10 --field 2 file.tsv`

2. Bin a column, aligning bins to start at 5
   `tva bin --width 10 --min 5 --field 2 file.tsv`

3. Bin a named column (requires header)
   `tva bin --header --width 0.5 --field score file.tsv`

4. Bin a column and append as new column
   `tva bin --header --width 10 --field Price --new-name Price_bin file.tsv`
