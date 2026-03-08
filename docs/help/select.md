# select

Selects and reorders TSV fields.

Behavior:
*   One of `--fields`/`-f` or `--exclude`/`-e` is required.
*   `--fields`/`-f` keeps only the listed fields, in the order given.
*   `--exclude`/`-e` drops the listed fields and keeps all others.
*   Use `--rest` to control where unlisted fields appear in the output.

Input:
*   Reads from files or standard input.
*   Files ending in `.gz` are transparently decompressed.

Output:
*   By default, output is written to standard output.
*   Use `--outfile` to write to a file instead.

Header behavior:
*   Supports `--header` / `-H` and `--header-hash1` modes.
*   `--header` / `-H`: Treats the first line of the input as a header (even if empty).
    The header is written once at the top of the output.
*   `--header-hash1`: Treats consecutive '#' lines plus the next line as header.
    The next line (column names) is written once at the top of the output.
*   In header mode, field names from the header can be used in field lists.

Field syntax:
*   Field lists support 1-based indices, ranges (`1-3,5-7`), header names, name
    ranges (`run-user_time`), and wildcards (`*_time`).
*   Run `tva --help-fields` for a full description shared across tva commands.

Examples:
1.  Select by name:
    `tva select input.tsv -H -f Name,Age`

2.  Select by index:
    `tva select input.tsv -f 1,3`

3.  Exclude columns:
    `tva select input.tsv -H -e Password,SSN`
