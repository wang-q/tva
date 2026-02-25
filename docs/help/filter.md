# filter

Filters TSV rows by field-based tests.

## Input

*   Reads from files or standard input; multiple files are processed as one stream.
*   Files ending in `.gz` are transparently decompressed.

## Header behavior

*   `--header` / `-H`: Treats the first non-empty line of the input as a header. The header is written once at the top of the output. Tests are applied only to data rows.

## Tests and logic

*   Multiple tests can be specified. By default, all tests must pass (logical AND).
*   Use `--or` to require that at least one test passes (logical OR).
*   Use `--invert` to invert the overall match result (select non-matching rows).
*   Use `--count` to print only the number of matching data rows.

## Field syntax

*   All tests that take a `<field-list>` argument accept the same field list syntax as other tva commands: 1-based indices, ranges, header names, name ranges, and wildcards.
*   Run `tva --help-fields` for a full description shared across tva commands.
