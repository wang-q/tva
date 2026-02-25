# append

Concatenates tab-separated values (TSV) files, similar to Unix `cat`, but with
header awareness and optional source tracking.

Header behavior:
*   --header / -H
    Treats the first line of each input as a header. Only the header from the
    first input is written; later headers are skipped.

Source tracking:
*   --track-source / -t
    Adds a column containing the source name for each data row. For regular
    files, the source name is the file name without extension. For standard
    input, the source name is `stdin`.
*   --source-header / -s STR
    Sets the header for the source column. Implies --header and --track-source.
    Default header name is `file`.
*   --file / -f LABEL=FILE
    Reads FILE and uses LABEL as the source value. Implies --track-source.

Delimiter:
*   --delimiter / -d CHR
    Field delimiter to use when adding the source column. Default: TAB.

Input:
*   If no input files or --file mappings are given, data is read from stdin.
*   Files ending in '.gz' are transparently decompressed.

Output:
*   By default, output is written to standard output.
*   Use --outfile to write to a file instead.
