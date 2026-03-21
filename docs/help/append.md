# append

Concatenates tab-separated values (TSV) files, similar to Unix `cat`, but with
header awareness and optional source tracking.

Input:

* Reads from files or standard input.
* Files ending in `.gz` are transparently decompressed.

Header behavior:

* Supports `--header` / `-H` and `--header-hash1` modes.
* When using header mode with multiple files, only the header from the first file is
  written; headers from subsequent files are skipped.

Source tracking:

* `--track-source` / `-t`: Adds a column containing the source name for each data row.
  For regular files, the source name is the file name without extension.
  For standard input, the source name is `stdin`.
* `--source-header` / `-s STR`: Sets the header for the source column.
  Implies `--header` and `--track-source`. Default header name is `file`.
* `--file` / `-f LABEL=FILE`: Reads FILE and uses LABEL as the source value.
  Implies `--track-source`.

Output:

* By default, output is written to standard output.
* Use `--outfile` to write to a file instead.

Examples:

1. Concatenate multiple files with header
   `tva append -H file1.tsv file2.tsv file3.tsv`

2. Track source file for each row
   `tva append -H -t file1.tsv file2.tsv`

3. Use custom source labels
   `tva append -H -f A=file1.tsv -f B=file2.tsv`
