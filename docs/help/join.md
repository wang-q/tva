# join

Joins lines from a TSV data stream against a filter file using one or more key
fields.

Input:
*   The filter file is specified with --filter-file and is read into memory.
*   Data is read from files or standard input.
*   Files ending in '.gz' are transparently decompressed.

Keys:
*   --key-fields/-k selects key fields from the filter file.
    *   Default: 0 (use entire line as the key).
*   --data-fields/-d selects key fields from the data stream, if different from
    --key-fields.
*   Field lists support numeric indices and, with --header, field names and
    ranges.

Output:
*   Matching lines from the data stream are written to standard output.
*   When --append-fields/-a is given, the selected fields from the filter file
    are appended to each matching data line.
*   When --header is set, exactly one header line is written, with any appended
    filter fields added to the data header.

Field syntax:
*   Field lists support 1-based indices, ranges (1-3,5-7), header names, name
    ranges (run-user_time), and wildcards (*_time).
*   Run `tva --help-fields` for a full description shared across tva commands.
