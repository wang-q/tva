# sort

Sorts TSV records by one or more keys.

Behavior:

* By default, comparisons are lexicographic.
* With `-n`/`--numeric`, comparisons are numeric (floating point).
* With `-r`/`--reverse`, the final ordering is reversed.
* Empty fields compare as empty strings in lexicographic mode and as 0 in
  numeric mode.

Input:

* Reads from files or standard input.
* Files ending in `.gz` are transparently decompressed.

Output:

* By default, output is written to standard output.
* Use `--outfile` to write to a file instead.

Header behavior:

* Supports all four header modes. See `tva --help-headers` for details.
* When header is enabled, header lines are preserved at the top of the output.

Field syntax:

* Use `-k`/`--key` to specify 1-based field indices or ranges (e.g., `2`, `4-5`).
* Multiple keys are supported and are applied in the order given.
* Run `tva --help-fields` for a full description shared across tva commands.

Examples:

1. Sort by first column
   `tva sort -k 1 file.tsv`

2. Sort numerically by second column
   `tva sort -k 2 -n file.tsv`

3. Sort by multiple columns
   `tva sort -k 1,2 file.tsv`

4. Sort in reverse order
   `tva sort -k 1 -r file.tsv`
