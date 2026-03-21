# slice

Slice rows by index (keep or drop).

Behavior:

* Selects specific rows by 1-based index (Keep Mode) or excludes them (Drop Mode).
* Row indices refer to absolute line numbers (including header lines when header mode is enabled).
* Range syntax:
    * `N` - Single row (e.g., `5`).
    * `N-M` - Row range from N to M (e.g., `10-20`).
    * `N-` - From row N to end of file (e.g., `10-`).
    * `-M` - From row 1 to row M (e.g., `-5` is equivalent to `1-5`).
* Multiple ranges can be specified with multiple `-r`/`--rows` flags.
* Use `--invert` to drop selected rows instead of keeping them.

Input:

* Reads from files or standard input.
* Files ending in `.gz` are transparently decompressed.

Output:

* By default, output is written to standard output.
* Use `--outfile` to write to a file instead.

Header behavior:

* Supports all four header modes. See `tva --help-headers` for details.
* When header is enabled, header lines are preserved in the output.

Examples:

1. Keep rows 10 to 20
   `tva slice -r 10-20 file.tsv`

2. Keep first 5 rows
   `tva slice -r -5 file.tsv`

3. Drop row 5 (exclude it)
   `tva slice -r 5 --invert file.tsv`

4. Preview with header (keep rows 100-110 plus header)
   `tva slice -H -r 100-110 file.tsv`
