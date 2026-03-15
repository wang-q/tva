# filter

Filters TSV rows by field-based tests.

Input:

* Reads from files or standard input; multiple files are processed as one stream.
* Files ending in `.gz` are transparently decompressed.

Tests and logic:

* Multiple tests can be specified. By default, all tests must pass (logical AND).
* Use `--or` to require that at least one test passes (logical OR).
* Use `--invert` to invert the overall match result (select non-matching rows).
* Use `--count` to print only the number of matching data rows.

Labeling:

* Use `--label` to add a column indicating whether each row passed the filter tests.
* Use `--label-values` to customize the pass/fail values (format: `PASS:FAIL`, default: `1:0`).
* When no tests are specified, all rows are considered passing.
* This is useful for adding a constant column to all rows.

Field syntax:

* All tests that take a `<field-list>` argument accept the same field list
  syntax as other tva commands: 1-based indices, ranges, header names, name
  ranges, and wildcards.
* Run `tva --help-fields` for a full description shared across tva commands.

Examples:

1. Filter rows where column 2 is greater than 100:
   `tva filter data.tsv --gt 2:100`

2. Add a 'year' column with value '2021' to all rows:
   `tva filter data.tsv -H --label year --label-values 2021:any`

3. Label rows as 'pass'/'fail' based on filter tests:
   `tva filter data.tsv -H --label status --label-values pass:fail --gt score:60`
