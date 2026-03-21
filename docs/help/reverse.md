# reverse

Reverses the order of lines (like tac).

Behavior:

* Reads all lines into memory. Large files may exhaust memory.
* Supports plain text and gzipped (`.gz`) TSV files.

Input:

* Reads from files or standard input.
* Files ending in `.gz` are transparently decompressed.

Output:

* By default, output is written to standard output.
* Use `--outfile` to write to a file instead.

Header behavior:

* Supports `--header` / `-H` (FirstLine mode) and `--header-hash1` (HashLines1 mode).
  See `tva --help-headers` for details.
* The header is written once at the top of the output, followed by reversed data lines.

Examples:

1. Reverse a file
   `tva reverse file.tsv`

2. Reverse a file, keeping the header at the top
   `tva reverse --header file.tsv`

3. Reverse a file with hash comment lines and column names
   `tva reverse --header-hash1 file.tsv`
