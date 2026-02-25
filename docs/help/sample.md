# sample

Samples or shuffles tab-separated values (TSV) rows using simple random algorithms.

## Modes

*   **Default shuffle**: With no sampling options, all input data rows are read and written in random order.
*   **Fixed-size sampling (`--num`/`-n`)**: Selects a random sample of N data rows and writes them in random order.
*   **Bernoulli sampling (`--prob`/`-p`)**: For each data row, independently includes the row in the output with probability PROB (0.0 < PROB <= 1.0). Row order is preserved.

## Header behavior

*   `--header` / `-H`: Treats the first non-empty line of the input as a header. The header is always written once at the top of the output. Sampling and shuffling are applied only to the remaining data rows.

## Input

*   If no input files are given, or an input file is 'stdin', data is read from standard input.
*   Files ending in '.gz' are transparently decompressed.

## Output

*   By default, output is written to standard output.
*   Use `--outfile` to write to a file instead.

## Field syntax

*   `--key-fields`/`-k` and `--weight-field`/`-w` accept the same field list syntax as other tva commands: 1-based indices, ranges, header names, name ranges, and wildcards.
*   Run `tva --help-fields` for a full description shared across tva commands.

## Random value printing

*   Use `--print-random` to prepend a random value column to sampled rows.
*   Use `--gen-random-inorder` to generate random values for all rows without changing input order.
