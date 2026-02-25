# reverse

Reverses the order of lines (like tac).

## Notes

*   Reads all lines into memory. Large files may exhaust memory.
*   Supports plain text and gzipped (`.gz`) TSV files.
*   Reads from stdin if no input file is given.

## Examples

1. Reverse a file:
   `tva reverse file.tsv`

2. Reverse a file, keeping the header at the top:
   `tva reverse --header file.tsv`
