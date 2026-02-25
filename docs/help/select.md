# select

Reads TSV data from files or standard input and writes selected fields to
standard output.

Fields can be specified by number or, when a header line is present, by field
name. Field numbers are 1-based and support ranges, for example `1,3-5`. When
`--header` is set, field names from the first header line can be used in the
field list.

Input:
*   If no input files are given, or an input file is `stdin`, data is read from
    standard input.
*   Files ending in `.gz` are transparently decompressed.

Selection:
*   One of `--fields`/`-f` or `--exclude`/`-e` is required.
*   `--fields`/`-f` keeps only the listed fields, in the order given.
*   `--exclude`/`-e` drops the listed fields and keeps all others.
*   In header mode, field names and numeric indices can be mixed.

Field Syntax:
*   Field lists support 1-based indices, ranges (`1-3,5-7`), header names, name
    ranges (`run-user_time`), and wildcards (`*_time`).
*   Run `tva --help-fields` for a full description shared across tva commands.

Output:
*   By default, output is written to standard output.
*   Use `--outfile` to write to a file instead.

Examples:
1. Select by name:
   `tva select input.tsv -H -f Name,Age`

2. Select by index:
   `tva select input.tsv -f 1,3`

3. Exclude columns:
   `tva select input.tsv -H -e Password,SSN`
