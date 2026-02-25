# check

Validates the structure of TSV input by ensuring that all lines have the
same number of fields.

Input:
*   If no input files are given, or an input file is 'stdin', data is read
    from standard input.
*   Files ending in '.gz' are transparently decompressed.

Behavior:
*   The number of fields on the first line is used as the expected count.
*   Each subsequent line must have the same number of fields.
*   On mismatch, details about the failing line and expected field count are
    printed to stderr and the command exits with a non-zero status.

Output:
*   On success, prints: '<N> lines, <M> fields'
