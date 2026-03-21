# md

Converts a tab-separated values (TSV) file into a markdown table.

Input:

* Reads from files or standard input.
* Files ending in `.gz` are transparently decompressed.

Output:

* Outputs a markdown formatted table.
* By default, output is written to standard output.

Formatting:

* `--num`: Right-align numeric columns automatically.
* `--fmt`: Format numeric columns with thousands separators and fixed decimals.
* `--digits`: Number of decimal digits to display (default: 0).
* `--center` / `-c`: Specify columns to center-align.
* `--right` / `-r`: Specify columns to right-align.

Examples:

1. Basic markdown table with numeric column right-aligned
   `tva to md data.tsv --num -c 2`

2. Formatted numeric columns with 2 decimal places
   `tva to md data.tsv --fmt --digits 2`
