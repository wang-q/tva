# md

Converts a tab-separated values (TSV) file into a markdown table.

Notes:
*   Supports plain text and gzipped (.gz) TSV files
*   Reads from stdin if input file is 'stdin'
*   With `--fmt`, numeric columns are formatted with thousands separators and
    fixed decimals
*   With `--num`, numeric columns are right-aligned automatically

Examples:
1.  Basic markdown table
    tva md tests/genome/ctg.range.tsv --num -c 2

2.  Formatted numeric columns
    tva md tests/genome/ctg.range.tsv --fmt --digits 2
