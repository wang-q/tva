# split

Splits TSV rows into multiple output files.

Behavior:

* Line count mode (`--lines-per-file`/`-l`): Writes a fixed number of data rows
    to each output file before starting a new one.
* Random assignment (`--num-files`/`-n`): Assigns each data row to one of N
    output files using a pseudo-random generator.
* Random assignment by key (`--num-files`/`-n`, `--key-fields`/`-k`): Uses
    selected fields as a key so that all rows with the same key are written to
    the same output file.
* Files are written to the directory given by `--dir` (default: current directory).
* File names are formed as: `<prefix><index><suffix>`.
* By default, existing files are rejected; use `--append`/`-a` to append to them.

Input:

* Reads from files or standard input.
* Files ending in `.gz` are transparently decompressed.

Output:

* Files are written to the directory specified by `--dir`.
* By default, output files are named `<prefix><index><suffix>`.

Header behavior:

* `--header-in-out`/`-H`: Treats the first line as header and writes it to every
    output file. The header is not counted against `--lines-per-file`.
* `--header-in-only`/`-I`: Treats the first line as header and does NOT write
    it to output files.

Field syntax:

* `--key-fields`/`-k` accepts 1-based field indices and ranges (e.g., `1,3-5`).
* Run `tva --help-fields` for a full description shared across tva commands.

Examples:

1. Split into files with 1000 lines each
   `tva split -l 1000 data.tsv --dir output/`

2. Randomly assign rows to 5 files
   `tva split -n 5 data.tsv --dir output/`

3. Split by key field (same key goes to same file)
   `tva split -n 5 -k 1 data.tsv --dir output/`
