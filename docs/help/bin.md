# bin

Discretizes numeric values into bins. Useful for creating histograms or grouping
continuous data.

Behavior:
*   Replaces the value in the target field with the bin start (lower bound).
*   Formula: `floor((value - min) / width) * width + min`.
*   Use `--new-name` to append as a new column instead of replacing.
*   Commonly used with `stats --groupby` to compute statistics per bin.

Input:
*   Reads from files or standard input.
*   Files ending in `.gz` are transparently decompressed.

Header behavior:
*   Supports `--header` / `-H` and `--header-hash1` modes.
*   `--header` / `-H`: Treats the first line of the input as a header (even if empty).
    The header is written once at the top of the output.
*   `--header-hash1`: Treats consecutive '#' lines plus the next line as header.
    The next line (column names) is written once at the top of the output.
*   When using multiple files with header mode enabled, the header from the first file is
    used, and headers from subsequent files are skipped.

Field syntax:
*   The `--field` argument accepts a 1-based index or a header name (when using
    `--header`).

Examples:
1.  Bin a numeric column with width 10:
    `tva bin --width 10 --field 2 file.tsv`

2.  Bin a column, aligning bins to start at 5:
    `tva bin --width 10 --min 5 --field 2 file.tsv`

3.  Bin a named column (requires header):
    `tva bin --header --width 0.5 --field score file.tsv`

4.  Bin a column and append as new column:
    `tva bin --header --width 10 --field Price --new-name Price_bin file.tsv`
