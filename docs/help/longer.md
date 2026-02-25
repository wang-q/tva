# longer

Reshapes a table from wide to long format by gathering multiple columns into
key-value pairs. This command is useful for "tidying" data where some column
names are actually values of a variable.

Input:
*   Reads from one or more TSV files or standard input.
*   Files ending in '.gz' are transparently decompressed.
*   The first line is ALWAYS treated as a header.
*   When multiple files are provided, the first file's header determines the
    schema (columns to reshape). Subsequent files must have the same column
    structure; their headers are skipped.

Reshaping behavior:
*   `--cols` / `-c`: Specifies which columns to reshape (melt). Can be column
    names, indices (1-based), or ranges (e.g., '3-5').
*   `--names-to`: The name of the new column that will contain the original
    column headers.
*   `--values-to`: The name of the new column that will contain the data values.
*   `--values-drop-na`: If set, rows where the value is empty will be omitted
    from the output.
*   `--names-prefix`: A string to remove from the start of each variable name.

Examples:
1. Reshape columns 3, 4, and 5 into default "name" and "value" columns:
   `tva longer data.tsv --cols 3-5`

2. Reshape columns starting with "wk", specifying new column names:
   `tva longer data.tsv --cols "wk*" --names-to week --values-to rank`

3. Reshape all columns except the first two:
   `tva longer data.tsv --cols 3-`

4. Process multiple files and save to output:
   `tva longer data1.tsv data2.tsv --cols 2-5 --outfile result.tsv`
