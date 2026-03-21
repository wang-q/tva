# longer

Reshapes a table from wide to long format by gathering multiple columns into
key-value pairs. This command is useful for "tidying" data where some column
names are actually values of a variable.

Behavior:

* Converts wide-format data to long format by melting specified columns.
* ID columns (those not specified in `--cols`) are preserved and repeated for each melted row.
* The first line is always treated as a header.
* When multiple files are provided, the first file's header determines the schema.
* Subsequent files must have the same column structure; their headers are skipped.
* Output is produced in row-major order (all melted rows for each input row are output together).

Input:

* Reads from one or more TSV files or standard input.
* Files ending in `.gz` are transparently decompressed.
* The first line is ALWAYS treated as a header.
* When multiple files are provided, the first file's header determines the
  schema (columns to reshape). Subsequent files must have the same column
  structure; their headers are skipped.

Output:

* By default, output is written to standard output.
* Use `--outfile` / `-o` to write to a file instead.
* Output columns: ID columns + name column(s) + value column.

Column selection:

* `--cols` / `-c`: Specifies which columns to reshape (melt).
* Columns can be specified by 1-based indices, ranges (e.g., `3-5`), or names (with wildcards like `Q*`).
* All columns not specified in `--cols` become ID columns and are preserved.

Names transformation:

* `--names-to`: The name(s) of the new column(s) that will contain the original column headers.
  Multiple names can be specified when using `--names-sep` or `--names-pattern`.
* `--values-to`: The name of the new column that will contain the data values (default: "value").
* `--names-prefix`: A string to remove from the start of each variable name.
* `--names-sep`: A separator to split column names into multiple columns.
* `--names-pattern`: A regex with capture groups to extract parts of column names into separate columns.

Field syntax:

* Field lists support 1-based indices, ranges (`1-3,5-7`), header names, name
  ranges (`run-user_time`), and wildcards (`*_time`).
* Run `tva --help-fields` for a full description shared across tva commands.

Missing values:

* `--values-drop-na`: If set, rows where the value is empty will be omitted from the output.
* Note: Whitespace-only values are not considered empty and will not be dropped.

Examples:

1. Reshape columns 3, 4, and 5 into default "name" and "value" columns
   `tva longer data.tsv --cols 3-5`

2. Reshape columns starting with "wk", specifying new column names
   `tva longer data.tsv --cols "wk*" --names-to week --values-to rank`

3. Reshape all columns except the first two
   `tva longer data.tsv --cols 3-`

4. Process multiple files and save to output
   `tva longer data1.tsv data2.tsv --cols 2-5 --outfile result.tsv`

5. Split column names into multiple columns using separator
   `tva longer data.tsv --cols 2-5 --names-sep "_" --names-to type num`

6. Extract parts of column names using regex pattern
   `tva longer data.tsv --cols 2-3 --names-pattern "new_?(.*)_(.*)" --names-to diag gender`

7. Remove prefix from column names before using as values
   `tva longer data.tsv --cols 2-4 --names-prefix "Q" --names-to question`

8. Drop rows with empty values
   `tva longer data.tsv --cols 2-5 --values-drop-na`
