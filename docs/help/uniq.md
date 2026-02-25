# uniq

Deduplicates rows of one or more tab-separated values (TSV) files without
sorting.

Input:
*   If no input files are given, or an input file is 'stdin', data is read
    from standard input.
*   Files ending in '.gz' are transparently decompressed.

Behavior:
*   Keeps a 64-bit hash for each unique key; ~8 bytes of memory per unique row.
*   Only the first occurrence of each key is kept; occurrences are not counted.

Field Syntax:
*   When `--header` is given, `--fields`/`-f` accepts 1-based indices, ranges
    (`1-3,5-7`), header names, name ranges (`run-user_time`), and wildcards
    (`*_time`).
*   Run `tva --help-fields` for a full description shared across tva commands.

Examples:
1.  Deduplicate whole rows:
    `tva uniq tests/genome/ctg.tsv`

2.  Deduplicate by column 2:
    `tva uniq tests/genome/ctg.tsv -f 2`
