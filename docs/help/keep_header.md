# keep-header

Runs an external command in a header-aware fashion. The first line of each
input file is treated as a header. The first header line is written to standard
output unchanged. All remaining lines (from all files) are sent to the given
command via standard input, excluding header lines from subsequent files. The
output produced by the command is appended after the initial header line.

Behavior:

* Preserves the specified number of header lines from the first non-empty input file.
* Header lines from subsequent files are skipped (only data lines are processed).
* The command is run with its standard input connected to the concatenated data lines
  (all lines after the header lines from each file).
* The command's standard output and standard error are passed through to this process.
* If no input files are given, data is read from standard input.

Input:

* Reads from files or standard input.
* Files ending in `.gz` are transparently decompressed.
* Use `-` to explicitly read from standard input.

Output:

* Header lines are written directly to standard output.
* Command output is appended after the header.
* Command exit code is propagated (non-zero exit codes are passed through).

Header behavior:

* `--lines` / `-n`: Number of header lines to preserve from the first non-empty input (default: 1).
* If set to 0, it defaults to 1.

Command execution:

* Usage: `tva keep-header [OPTIONS] [FILE...] -- <COMMAND> [ARGS...]`
* A double dash (`--`) **must** be used to separate input files from the command to run,
  similar to how the pipe operator (`|`) separates commands in a shell pipeline.
* The command is required and must be specified after `--`.
* The command receives all data lines (excluding headers) on its standard input.
* The command's standard output and standard error streams are passed through unchanged.

Examples:

1. Sort a file while keeping the header line first
   `tva keep-header data.tsv -- sort`

2. Sort multiple TSV files numerically on field 2, preserving one header
   `tva keep-header data1.tsv data2.tsv -- sort -t $'\t' -k2,2n`

3. Read from stdin, filter with grep, and keep the original header
   `cat data.tsv | tva keep-header -- grep red`

4. Preserve multiple header lines
   `tva keep-header --lines 2 data.tsv -- sort`
