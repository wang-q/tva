# to csv

Converts TSV input to CSV format.

Behavior:

* Converts TSV data into CSV format.
* Fields containing delimiters, quotes, or newlines are properly escaped
    and quoted according to the CSV specification.
* Use `--delimiter` to specify a custom CSV field delimiter (default: comma).

Input:

* Reads from files or standard input.
* Files ending in `.gz` are transparently decompressed.

Output:

* By default, output is written to standard output.
* Use `--outfile` to write to a file instead.

Examples:

1. Convert a TSV file to CSV
   `tva to csv data.tsv > data.csv`

2. Read from stdin and convert to CSV
   `cat data.tsv | tva to csv > data.csv`

3. Use a custom delimiter
   `tva to csv --delimiter ';' data.tsv > data.csv`
