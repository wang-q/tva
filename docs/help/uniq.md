# uniq

Deduplicates TSV rows from one or more files without sorting.

Behavior:

* Keeps a 64-bit hash for each unique key; ~8 bytes of memory per unique row.
* Only the first occurrence of each key is kept by default.
* Use `--repeated` / `-r` to output only lines that are repeated.
* Use `--at-least` / `-a` to output only lines repeated at least N times.
* Use `--max` / `-m` to limit the number of occurrences output per key.
* Use `--equiv` / `-e` to append equivalence class IDs.
* Use `--number` / `-z` to append occurrence numbers for each key.

Input:

* Reads from files or standard input.
* Files ending in `.gz` are transparently decompressed.

Output:

* By default, output is written to standard output.
* Use `--outfile` / `-o` to write to a file instead.

Header behavior:

* Supports `--header` / `-H` and `--header-hash1` modes.
* When using header mode with multiple files, only the header from the first file is
  written; headers from subsequent files are skipped.

Field syntax:

* Use `--fields` / `-f` to specify columns to use as the deduplication key.
* Use `0` to indicate the entire line should be used as the key (default behavior).
* Field lists support 1-based indices, ranges (`1-3,5-7`), header names, name
  ranges (`run-user_time`), and wildcards (`*_time`).
* Run `tva --help-fields` for a full description shared across tva commands.

Examples:

1. Deduplicate whole rows
   `tva uniq data.tsv`

2. Deduplicate by column 2
   `tva uniq data.tsv -f 2`

3. Deduplicate with header using named fields
   `tva uniq --header -f name,age data.tsv`

4. Output only repeated lines
   `tva uniq --repeated data.tsv`

5. Output lines repeated at least 3 times
   `tva uniq --at-least 3 data.tsv`

6. Output with equivalence class IDs
   `tva uniq --header -f 1 --equiv --number data.tsv`

7. Deduplicate multiple files with header
   `tva uniq --header file1.tsv file2.tsv file3.tsv`

8. Ignore case when comparing
   `tva uniq --ignore-case data.tsv`
