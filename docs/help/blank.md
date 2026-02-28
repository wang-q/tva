# blank

Replaces consecutive identical values in selected columns with a blank string (or a custom value).

Input:
*   Reads from files or standard input; multiple files are processed as one stream.
*   Files ending in `.gz` are transparently decompressed.

Header behavior:
*   `--header` / `-H`: Treats the first line of the input as a header.
    The header is written once at the top of the output. Blanking logic starts
    from the first data row.
*   If no header is specified, the first line is treated as data.

Blanking logic:
*   For each selected column, the current value is compared with the value in the
    previous row.
*   If the values are identical, the current cell is replaced with an empty string
    (or the specified replacement value).
*   If the values differ, the current value is written, and it becomes the new
    reference for subsequent rows.
*   Blanking is stateful across file boundaries when multiple files are provided.
*   Use `-i` / `--ignore-case` to compare values case-insensitively.

Field Syntax:
*   Use `-f` / `--field` to specify columns to blank.
*   Format: `COL` (blank with empty string) or `COL:REPLACEMENT` (blank with custom string).
*   Columns can be specified by 1-based index or, if `-H` is used, by header name.
*   Run `tva --help-fields` for a full description shared across tva commands.

Output:
*   Writes processed records to standard output or to the file given by `--outfile`.

Examples:
1. Blank the first column:
   `tva blank -H -f 1 data.tsv`

2. Blank the 'category' column with "---":
   `tva blank -H -f category:--- data.tsv`

3. Blank multiple columns:
   `tva blank -H -f 1 -f 2 data.tsv`
