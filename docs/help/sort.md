# sort

Sorts TSV/CSV records by one or more keys.

Input:
*   If no input files are given, or an input file is 'stdin', data is read from
    standard input.
*   Files ending in `.gz` are transparently decompressed.

Keys:
*   Use `-k`/`--key` to specify 1-based field indices or ranges (for example:
    `2`, `4-5`).
*   Multiple keys are supported and are applied in the order given.

Behavior:
*   By default, comparisons are lexicographic.
*   With `-n`/`--numeric`, comparisons are numeric (floating point).
*   With `-r`/`--reverse`, the final ordering is reversed.
*   For an MxN table, the output contains the same rows sorted by the selected
    key fields.
*   Empty fields compare as empty strings in lexicographic mode and as 0 in
    numeric mode.

Output:
*   Writes sorted records to standard output or to the file given by `--outfile`.
