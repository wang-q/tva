# tva

Tab-separated Values Assistant
Fast, reliable TSV processing toolkit in Rust

[![Build](https://github.com/wang-q/tva/actions/workflows/build.yml/badge.svg)](https://github.com/wang-q/tva/actions)
[![codecov](https://codecov.io/gh/wang-q/tva/branch/master/graph/badge.svg?token=8toyNHCsVU)](https://codecov.io/gh/wang-q/tva)
[![license](https://img.shields.io/github/license/wang-q/tva)](https://github.com//wang-q/tva)

## Synopsis

### `tva help`

```text
tva: Tab-separated Values Assistant

Usage: tva [COMMAND]

Commands:
  md     Converts TSV file to markdown table
  dedup  Deduplicates TSV rows from one or more files
  nl     Adds line numbers to TSV rows
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version


Description:
Tab-separated Values Assistant with subcommands for working with TSV files.

Subcommand groups:
* Generic TSV: md, dedup, nl

Notes:
* Run `tva help <SUBCOMMAND>` for detailed usage
```

## Examples

```bash
tva md tests/genome/ctg.range.tsv --num -c 2
tva md tests/genome/ctg.range.tsv --fmt --digits 2

tva dedup tests/genome/ctg.tsv tests/genome/ctg.tsv
tva dedup tests/genome/ctg.tsv -f 2

tva nl tests/genome/ctg.tsv

```

## Help text style guide

* **`about`**: Third-person singular, describing the TSV operation
  (e.g., "Converts TSV to markdown table", "Deduplicates TSV rows").
* **`after_help`**: Use raw string `r###"..."###`.
    * **Description**: Short paragraph of what the subcommand does and its trade-offs.
    * **Notes**: Bullet points starting with `*`.
        * TSV input: `* Supports plain text and gzipped (.gz) TSV files`
        * Stdin behavior:
            * Single-input tools (e.g. `md`): `* Reads from stdin if input file is 'stdin' or no input file is given`
            * Multi-input tools (e.g. `dedup`): `* Reads from stdin if no input file is given or if input file is 'stdin'`
        * Memory-heavy tools (e.g. `dedup`): `* Keeps a hash for each unique row; does not count occurrences`
    * **Examples**: Numbered list (`1.`, `2.`) with code blocks indented by 3 spaces.
* **Arguments**:
    * **Input**: `infile` (single) or `infiles` (multiple).
        * Help (single): `Input TSV file to process (default: stdin).`
        * Help (multiple): `Input TSV file(s) to process`.
    * **Output**: `outfile` (`-o`, `--outfile`).
        * Help: `Output filename. [stdout] for screen`.
* **Terminology**:
    * Prefer "TSV" when referring to files.
    * Use "row" / "column" in help text where it makes sense.

## Author

Qiang Wang <wang-q@outlook.com>

## License

MIT.

Copyright by Qiang Wang.

Written by Qiang Wang <wang-q@outlook.com>, 2026-
