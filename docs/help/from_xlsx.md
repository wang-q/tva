# from xlsx

Reads data from an Excel file (.xlsx) and writes it as TSV.

This command converts Excel spreadsheets into TSV format. It supports reading from
specific sheets and handles basic cell values.

Input:
*   An Excel file path is required.

Output:
*   Each row in the spreadsheet becomes one TSV line.
*   Cells are joined with TAB characters.

Examples:
1.  Convert an Excel file to TSV (first sheet)
    tva from xlsx data.xlsx > data.tsv

2.  Read a specific sheet
    tva from xlsx --sheet "Sheet2" data.xlsx > data.tsv
