# slice

Slice rows by index (1-based). Can be used to select specific rows (Keep Mode)
or exclude them (Drop Mode).

Notes:
*   Supports plain text and gzipped (`.gz`) TSV files.
*   Reads from stdin if no input file is given.
*   Row indices are 1-based.
*   Multiple ranges can be specified with multiple `-r`/`--rows` flags.

Examples:
1. Keep rows 10 to 20:
   `tva slice -r 10-20 file.tsv`

2. Keep rows 1-5 and 10-15:
   `tva slice -r 1-5 -r 10-15 file.tsv`

3. Drop row 5 (exclude it):
   `tva slice -r 5 --invert file.tsv`

4. Drop rows 1-5 (exclude header and first 4 data rows):
   `tva slice -r 1-5 --invert file.tsv`

5. Drop rows 2-5 but keep header (row 1):
   `tva slice -H -r 2-5 --invert file.tsv`

6. Preview with header (Keep rows 100-110 plus header):
   `tva slice -H -r 100-110 file.tsv`
