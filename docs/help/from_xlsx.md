# from xlsx

Converts Excel (.xlsx/.xls) input to TSV output.

Behavior:

* Reads data from Excel spreadsheets and converts each row to a TSV line.
* By default, reads from the first sheet in the workbook.
* Use `--sheet` to specify a sheet by name.
* Use `--list-sheets` to list all available sheet names.
* Cell values are converted to strings:
    * Empty cells become empty strings.
    * TAB, newline, and carriage return characters are replaced with spaces.

Input:

* Requires an Excel file path (.xlsx or .xls).

Output:

* Each spreadsheet row becomes one TSV line.
* Cells are joined with TAB characters.
* By default, output is written to standard output.
* Use `--outfile` to write to a file instead.

Examples:

1. Convert an Excel file to TSV (first sheet)
   `tva from xlsx data.xlsx > data.tsv`

2. Convert a specific sheet by name
   `tva from xlsx --sheet "Sheet2" data.xlsx`

3. List all sheet names in a workbook
   `tva from xlsx --list-sheets data.xlsx`
