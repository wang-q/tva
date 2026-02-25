# wider

Reshapes a table from long to wide format by spreading a key-value pair across
multiple columns. This is the inverse of 'longer' and similar to 'crosstab'.

Input:
*   Reads from one or more TSV files or standard input.
*   Files ending in '.gz' are transparently decompressed.
*   The first line is ALWAYS treated as a header.
*   When multiple files are provided, they must have the SAME column structure.

Reshaping behavior:
*   `--names-from`: Column(s) containing the new column headers.
*   `--values-from`: Column(s) containing the data values. Required unless op is
    'count'.
*   `--id-cols`: Columns that identify each row. If omitted, all columns except
    'names-from' and 'values-from' are used.
*   `--values-fill`: Value to use for missing cells (default: empty).
*   `--names-sort`: Sort the resulting column headers alphabetically.
*   `--op`: Aggregation operation to perform when multiple values fall into the
    same cell. Default is 'last' (last value wins).

Examples:
1. Spread 'key' and 'value' columns back into wide format:
   `tva wider --names-from key --values-from value`

2. Spread 'measurement' column, using 'result' as values:
   `tva wider --names-from measurement --values-from result`

3. Specify ID columns explicitly (dropping others):
   `tva wider --names-from key --values-from val --id-cols id date`

4. Count occurrences (crosstab):
   `tva wider --names-from category --id-cols region --op count`

5. Calculate sum of values:
   `tva wider --names-from category --values-from amount --id-cols region --op sum`
