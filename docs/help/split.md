# split

Splits tab-separated values (TSV) rows into multiple output files.

Modes:
*   Line count mode (--lines-per-file/-l): Writes a fixed number of data rows to
    each output file before starting a new one.
*   Random assignment (--num-files/-n): Assigns each data row to one of N output
    files using a pseudo-random generator.
*   Random assignment by key (--num-files/-n, --key-fields/-k): Uses selected
    fields as a key so that all rows with the same key are written to the same
    output file.

Key fields:
*   --key-fields/-k accepts a numeric field list using 1-based indices and
    ranges (for example 1,3-5). The selected fields are concatenated to form
    the key that controls random assignment.

Field syntax:
*   See `tva --help-fields` for the shared field list syntax used across tva
    commands. Note that tva split currently only accepts numeric indices for
    --key-fields/-k; header names and wildcards are not yet supported here.

Header behavior:
*   --header-in-out / -H
    Treats the first non-empty line of the input as a header. The header is not
    counted against --lines-per-file and is written to every output file.

Output:
*   Files are written to the directory given by --dir (default: current
    directory).
*   File names are formed as: <prefix><index><suffix>, where <index> is a
    1-based counter optionally zero-padded to --digit-width digits.
*   By default, existing files are rejected; use --append/-a to append to them.
