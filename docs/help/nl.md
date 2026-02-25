# nl

Reads TSV data from files or standard input and writes each line preceded
by a line number. This is a simplified, TSV-aware version of the Unix
`nl` program and adds support for treating the first input line as a
header.

Supports plain text and gzipped (.gz) files. When multiple files are
given, lines are numbered continuously across files.

Input:
*   If no input files are given, or an input file is 'stdin', data is read
    from standard input.
*   Files ending in '.gz' are transparently decompressed.
*   Completely empty files (only blank lines) are skipped and do not consume
    line numbers.

Header behavior:
*   --header / -H
    Treats the first line of each file as a header. Only the header from
    the first non-empty file is written; later header lines are skipped
    and not numbered.
*   --header-string / -s
    Sets the header text for the line number column (default: 'line') and
    implies --header.

Numbering:
*   Line numbers start from --start-number / -n (default: 1, can be negative).
*   Numbers increase by 1 for each data line, across all input files.
*   Header lines are never numbered.

Examples:
1.  Number lines of a TSV file
    tva nl tests/genome/ctg.tsv

2.  Number lines with a header for the line number column
    tva nl --header --header-string LINENUM tests/genome/ctg.tsv

3.  Number lines starting from 100
    tva nl --start-number 100 tests/genome/ctg.tsv

4.  Number multiple files, preserving continuous line numbers
    tva nl input1.tsv input2.tsv

5.  Read from stdin
    cat input1.tsv | tva nl
