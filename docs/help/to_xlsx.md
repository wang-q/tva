# to xlsx

Converts TSV input to Excel (.xlsx) format.

Behavior:

* Creates an Excel spreadsheet from TSV data.
* Writes all input rows into a single sheet.
* Supports conditional formatting with `--le`, `--ge`, `--bt`, and `--str-in-fld`.
* Numeric fields are written as numbers; non-numeric fields are written as strings.

Input:

* Reads from files (stdin is not supported for binary xlsx output).
* Files ending in `.gz` are transparently decompressed.

Output:

* An Excel (.xlsx) file.
* Use `--outfile` to specify the output filename (default: `<infile>.xlsx`).
* Use `--sheet` to specify the sheet name (default: input file basename).

Header behavior:

* `--header` / `-H`: Treats the first non-empty row as header and styles it.
* When header is enabled, the header row is frozen in the output.

Examples:

1. Convert a TSV file to Excel
   `tva to xlsx data.tsv`

2. Specify output filename
   `tva to xlsx data.tsv --outfile output.xlsx`

3. Specify sheet name and header
   `tva to xlsx data.tsv --sheet "MyData" --header`

4. Apply conditional formatting
   `tva to xlsx data.tsv --header --le "2:100" --ge "3:50"`
