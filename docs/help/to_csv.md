# to csv

Reads TSV data and writes it as CSV.

This command converts TSV data into CSV format, ensuring that fields containing
delimiters, quotes, or newlines are properly escaped and quoted according to the
CSV specification.

Input:
*   TSV data from a file or standard input.

Output:
*   CSV data where fields are separated by a custom delimiter (default: ,).

Examples:
1.  Convert a TSV file to CSV
    tva to csv data.tsv > data.csv

2.  Read from stdin and convert to CSV
    cat data.tsv | tva to csv > data.csv

3.  Use a custom delimiter
    tva to csv --delimiter ';' data.tsv > data.csv
