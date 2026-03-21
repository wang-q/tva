# wider

Reshapes a table from long to wide format by spreading a key-value pair across
multiple columns. This is the inverse of `longer` and similar to `crosstab`.

Behavior:

* Converts long-format data to wide format by spreading columns.
* ID columns (specified by `--id-cols`) are preserved and identify each row.
* The `--names-from` column values become the new column headers.
* The `--values-from` column values populate the new columns.
* When multiple values map to the same cell, an aggregation operation is performed.
* Missing cells are filled with the value specified by `--values-fill` (default: empty).

Input:

* Reads from one or more TSV files or standard input.
* Files ending in `.gz` are transparently decompressed.
* The first line is ALWAYS treated as a header.
* When multiple files are provided, they must have the same column structure.

Output:

* By default, output is written to standard output.
* Use `--outfile` / `-o` to write to a file instead.

Header behavior:

* Supports `--header` / `-H` and `--header-hash1` modes.
* The first line is always treated as a header to resolve column names.

Field syntax:

* Use `--names-from` to specify the column containing new column headers.
* Use `--values-from` to specify the column containing data values.
* Use `--id-cols` to specify columns that identify each row.
* Field lists support 1-based indices, ranges (`1-3,5-7`), header names, name
  ranges (`run-user_time`), and wildcards (`*_time`).
* Run `tva --help-fields` for a full description shared across tva commands.

Examples:

1. Spread `key` and `value` columns back into wide format
    tva wider --names-from key --values-from value data.tsv

2. Spread `measurement` column, using `result` as values
    tva wider --names-from measurement --values-from result data.tsv

3. Specify ID columns explicitly (dropping others)
    tva wider --names-from key --values-from val --id-cols id,date data.tsv

4. Count occurrences (crosstab)
    tva wider --names-from category --id-cols region --op count data.tsv

5. Calculate sum of values
    tva wider --names-from category --values-from amount --id-cols region --op sum data.tsv

6. Fill missing values with custom string
    tva wider --names-from key --values-from val --values-fill "NA" data.tsv

7. Sort resulting column headers alphabetically
    tva wider --names-from key --values-from val --names-sort data.tsv
