# stats

Calculates summary statistics for TSV data.

Behavior:

* Supports various statistical operations: count, sum, mean, median, min, max,
    stdev, variance, mode, quantiles, and more.
* Use `--group-by` to calculate statistics per group.
* Multiple operations can be specified in a single command.

Input:

* Reads from files or standard input.
* Files ending in `.gz` are transparently decompressed.

Output:

* By default, output is written to standard output.
* Use `--write-header` to write an output header even if there is no input header.

Header behavior:

* Supports `--header` / `-H` and `--header-hash1` modes.
* In header mode, field names from the header can be used in field lists.

Field syntax:

* `--group-by`/`-g` and all operation flags accept 1-based field indices,
    ranges, header names, and wildcards.
* Run `tva --help-fields` for a full description shared across tva commands.

Examples:

1. Calculate basic stats for a column
   `tva stats docs/data/us_rent_income.tsv --header --mean estimate --max estimate`

2. Group by a column
   `tva stats docs/data/us_rent_income.tsv -H --group-by variable --mean estimate`

3. Count rows per group
   `tva stats docs/data/us_rent_income.tsv -H --group-by NAME --count`

4. List unique values in a group
   `tva stats docs/data/us_rent_income.tsv -H --group-by variable --unique estimate`

5. Pick a random value from a group
   `tva stats docs/data/us_rent_income.tsv -H --group-by variable --rand estimate`
