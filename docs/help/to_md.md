# to md

Converts TSV input to a markdown table.

Behavior:

* Outputs a markdown formatted table.
* Use `--num` to right-align numeric columns automatically.
* Use `--fmt` to format numeric columns with thousands separators and fixed decimals.
* Use `--center` / `-c` to specify columns to center-align.
* Use `--right` / `-r` to specify columns to right-align.
* Use `--digits` to set the number of decimal digits (default: 0).

Input:

* Reads from files or standard input.
* Files ending in `.gz` are transparently decompressed.

Output:

* By default, output is written to standard output.
* Use `--outfile` to write to a file instead.

Examples:

1. Basic markdown table with numeric column right-aligned:
   `tva to md data.tsv --num`

2. Center-align specific columns:
   `tva to md data.tsv --num --center 2`

3. Formatted numeric columns with 2 decimal places:
   `tva to md data.tsv --fmt --digits 2`
