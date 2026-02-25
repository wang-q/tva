# bin

Discretize numeric values into bins. Useful for creating histograms or grouping
continuous data.

Replaces the value in the target field with the bin start (lower bound).
Formula: `floor((value - min) / width) * width + min`

Notes:
*   Supports plain text and gzipped (`.gz`) TSV files.
*   Reads from stdin if no input file is given.
*   When using multiple files with `--header`, the header from the first file is
    used, and headers from subsequent files are skipped.

Examples:
1. Bin a numeric column with width 10:
   `tva bin --width 10 --field 2 file.tsv`

2. Bin a column, aligning bins to start at 5 (e.g., 5-15, 15-25):
   `tva bin --width 10 --min 5 --field 2 file.tsv`

3. Bin a named column (requires header):
   `tva bin --header --width 0.5 --field score file.tsv`

4. Bin a column and append as new column:
   `tva bin --header --width 10 --field Price --new-name Price_bin file.tsv`
