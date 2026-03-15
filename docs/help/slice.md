# slice

Slice rows by index (1-based). Can be used to select specific rows (Keep Mode)
or exclude them (Drop Mode).

Notes:

* Supports plain text and gzipped (`.gz`) TSV files.
* Reads from stdin if no input file is given.
* Row indices are 1-based.
* Multiple ranges can be specified with multiple `-r`/`--rows` flags.

Range syntax:

* `N` - Single row (e.g., `5` means row 5).
* `N-M` - Row range from N to M (e.g., `10-20`).
* `N-` - From row N to end of file (e.g., `10-`).
* `-M` - From row 1 to row M (e.g., `-5` is equivalent to `1-5`).

Header behavior:

* Supports all four header modes. See `tva --help-headers` for details.
* When header is enabled, header lines are preserved in the output and row
  indices refer to absolute line numbers (including header lines).
* For example, with `--header-lines 3`, the first data row is row 4.

Examples:

1. Keep rows 10 to 20:
   `tva slice -r 10-20 file.tsv`

2. Keep rows 1-5 and 10-15:
   `tva slice -r 1-5 -r 10-15 file.tsv`

3. Keep first 5 rows (shorthand):
   `tva slice -r -5 file.tsv`

4. Keep from row 10 to end:
   `tva slice -r 10- file.tsv`

5. Drop row 5 (exclude it):
   `tva slice -r 5 --invert file.tsv`

6. Drop rows 1-5 (exclude header and first 4 data rows):
   `tva slice -r 1-5 --invert file.tsv`

7. Drop rows 2-5 but keep header (row 1):
   `tva slice -H -r 2-5 --invert file.tsv`

8. Preview with header (Keep rows 100-110 plus header):
   `tva slice -H -r 100-110 file.tsv`

9. Keep rows with multi-line header (first 3 lines are header):
   `tva slice --header-lines 3 -r 5-10 file.tsv`
