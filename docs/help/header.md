# header

Prints the headers of TSV files in a transposed table format, with duplicated
and diverging column names indicated.

Behavior:

* Reads the first line of each input file as the header.
* Outputs headers in a transposed table format (files as columns, headers as rows).
* First column shows the column index (1-based by default, labeled "file").
* Marks duplicate column names within a file with `[duplicate]`.
* When comparing multiple files, marks headers not present in all files with `[diverging]`.

Input:

* Reads from files or standard input.
* Files ending in `.gz` are transparently decompressed.

Examples:

1. Display headers of a single file
   `tva header data.tsv`

2. Show only header names without indices
   `tva header -n data.tsv`

3. Compare headers across multiple files
   `tva header file1.tsv file2.tsv`

4. Start indices from 0
   `tva header -s 0 data.tsv`
