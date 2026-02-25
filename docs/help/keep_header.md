# keep-header

Runs an external command in a header-aware fashion. The first line of each
input file is treated as a header. The first header line is written to standard
output unchanged. All remaining lines (from all files) are sent to the given
command via standard input, excluding header lines from subsequent files. The
output produced by the command is appended after the initial header line.

Usage:
  tva keep-header [file...] -- program [args...]

Notes:
*   If no input files are given, data is read from standard input.
*   The number of header lines to preserve from the first non-empty input can be
    configured with --lines / -n (default: 1).
*   A double dash (--) separates input files from the command to run, similar
    to how the pipe operator (|) separates commands in a shell pipeline.
*   The command is run with its standard input connected to the concatenated
    data lines (all lines after the first header line of each file).
*   The command's standard output and standard error are passed through to
    this process.

Examples:
1.  Sort a file while keeping the header line first
    tva keep-header data.tsv -- sort

2.  Sort multiple TSV files numerically on field 2, preserving one header
    tva keep-header data1.tsv data2.tsv -- sort -t $'\t' -k2,2n

3.  Read from stdin, filter with grep, and keep the original header
    cat data.tsv | tva keep-header -- grep red
