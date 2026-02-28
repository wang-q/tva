# to xlsx

Reads TSV data and writes it as an Excel file (.xlsx).

This command allows you to create an Excel spreadsheet from TSV data. It writes
all input rows into a single sheet named "Sheet1".

Input:
*   TSV data from a file or standard input.

Output:
*   An Excel (.xlsx) file containing the data.

Examples:
1.  Convert a TSV file to Excel
    tva to xlsx data.tsv > data.xlsx

2.  Read from stdin and convert to Excel
    cat data.tsv | tva to xlsx > data.xlsx
