# transpose

Transposes a tab-separated values (TSV) table by swapping rows and columns.

Behavior:
*   Reads a single TSV input as a whole table and performs a matrix transpose.
*   Uses the number of fields in the first line as the expected width.
*   All subsequent lines must have the same number of fields.
*   On mismatch, an error is printed and the command exits with non-zero status.

Input:
*   If no input file is given, or the input file is 'stdin', data is read from
    standard input.
*   Files ending in `.gz` are transparently decompressed.

Output:
*   For an MxN matrix (M lines, N fields), writes an NxM matrix.
*   If the input is empty, no output is produced.

Notes:
*   This command only operates in strict mode; non-rectangular tables are
    rejected.
