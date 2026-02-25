# from-csv

Reads CSV data from a file or standard input and writes it as TSV.

The goal is to provide a simple entry point into the tva toolkit for
CSV sources. Parsing is delegated to the Rust `csv` crate, so quoted
fields, embedded delimiters, and newlines are handled according to the
CSV specification.

Input:
*   If no input file is given, or the input file is 'stdin', data is read
    from standard input.

Output:
*   Each CSV record becomes one TSV line.
*   Fields are joined with TAB characters.

Examples:
1.  Convert a CSV file to TSV
    tva from-csv data.csv > data.tsv

2.  Read CSV from stdin and convert to TSV
    cat data.csv | tva from-csv > data.tsv

3.  Use a custom delimiter
    tva from-csv --delimiter ';' data.csv > data.tsv
