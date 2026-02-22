tva

Tab-separated Values Assistant
A fast, reliable TSV processing toolkit written in Rust

[![Build](https://github.com/wang-q/tva/actions/workflows/build.yml/badge.svg)](https://github.com/wang-q/tva/actions)
[![codecov](https://codecov.io/gh/wang-q/tva/branch/master/graph/badge.svg?token=8toyNHCsVU)](https://codecov.io/gh/wang-q/tva)
[![license](https://img.shields.io/github/license/wang-q/tva)](https://github.com//wang-q/tva)

## Synopsis

### `tva help`

```text
`tva` Tab-separated Values Assistant

Usage: tva.exe [COMMAND]

Commands:
  md    Convert .tsv file to markdown table
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version


Subcommand groups:

* Generic .tsv: md
```

## Examples

```bash
cargo run --bin tva md tests/genome/ctg.range.tsv --num -c 2
cargo run --bin tva md tests/genome/ctg.range.tsv --fmt --digits 2

cargo run --bin tva dedup tests/genome/ctg.tsv tests/genome/ctg.tsv
cargo run --bin tva dedup tests/genome/ctg.tsv -f 2


```
