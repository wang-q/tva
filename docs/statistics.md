# Statistics Documentation

This document explains how to use the statistics and summary commands in `tva`: **`stats`**, **`bin`**, and **`uniq`**. These commands allow you to summarize data, discretize values, and deduplicate rows.

## Introduction

*   **`stats`**: Calculates summary statistics (like sum, mean, max) for fields, optionally grouping by key fields.
*   **`bin`**: Discretizes numeric values into bins (useful for histograms).
*   **`uniq`**: Deduplicates rows based on a key, with options for equivalence classes and occurrence numbering.

## `stats` (Summary Statistics)

The `stats` command calculates summary statistics for specified fields. It mimics the functionality of `tsv-summarize`.

### Basic Usage

```bash
tva stats [input_files...] [options]
```

### Options

*   **`--header` / `-H`**: Treat the first line of each file as a header.
*   **`--group-by` / `-g`**: Fields to group by (e.g., `1`, `1,2`).
*   **`--count` / `-c`**: Count the number of rows.
*   **`--sum`**: Calculate sum of fields.
*   **`--mean`**: Calculate mean of fields.
*   **`--min`**: Calculate min of fields.
*   **`--max`**: Calculate max of fields.
*   **`--median`**: Calculate median of fields.
*   **`--stdev`**: Calculate standard deviation of fields.
*   **`--variance`**: Calculate variance of fields.
*   **`--mad`**: Calculate median absolute deviation of fields.
*   **`--first`**: Get the first value of fields.
*   **`--last`**: Get the last value of fields.

### Examples

#### 1. Calculate basic stats for a column

Calculate the mean and max of the `estimate` column in `docs/data/us_rent_income.tsv`:

```bash
tva stats docs/data/us_rent_income.tsv --header --mean estimate --max estimate
```

Output:
```tsv
estimate_mean	estimate_max
14316.2	32940
```

#### 2. Group by a column

Group by `variable` and calculate the mean of `estimate`:

```bash
tva stats docs/data/us_rent_income.tsv --header --group-by variable --mean estimate
```

Output:
```tsv
variable	estimate_mean
income	27635.2
rent	997.2
```

#### 3. Count rows per group

Count the number of rows for each unique value in `NAME`:

```bash
tva stats docs/data/us_rent_income.tsv --header --group-by NAME --count
```

Output (first 5 lines):
```tsv
NAME	count
Alabama	2
Alaska	2
Arizona	2
Arkansas	2
```

## `bin` (Discretize Values)

The `bin` command discretizes numeric values into bins. This is useful for creating histograms or grouping continuous data.

### Basic Usage

```bash
tva bin [input_files...] --width <width> --field <field> [options]
```

### Options

*   **`--width` / `-w`**: Bin width (bucket size). Required.
*   **`--field` / `-f`**: Field to bin (1-based index or name). Required.
*   **`--min` / `-m`**: Alignment/Offset (bin start). Default: 0.0.
*   **`--new-name`**: Append as new column with this name (instead of replacing).
*   **`--header` / `-H`**: Input has header.

### Notes

*   Formula: `floor((value - min) / width) * width + min`
*   Replaces the value in the target field with the bin start (lower bound) unless `--new-name` is used.

### Examples

#### 1. Bin a numeric column

Bin the `breaks` column in `docs/data/warpbreaks.tsv` with a width of 10:

```bash
tva bin docs/data/warpbreaks.tsv --header --width 10 --field breaks
```

Output (first 5 lines):
```tsv
wool	tension	breaks
A	L	20
A	L	30
A	L	50
A	M	10
```

#### 2. Bin with alignment

Bin the `breaks` column, aligning bins to start at 5:

```bash
tva bin docs/data/warpbreaks.tsv --header --width 10 --min 5 --field breaks
```

Output (first 5 lines):
```tsv
wool	tension	breaks
A	L	25
A	L	25
A	L	45
A	M	15
```

#### 3. Append bin as a new column

Bin the `breaks` column and append the result as `breaks_bin`:

```bash
tva bin docs/data/warpbreaks.tsv --header --width 10 --field breaks --new-name breaks_bin
```

Output (first 5 lines):
```tsv
wool	tension	breaks	breaks_bin
A	L	26	20
A	L	30	30
A	L	54	50
A	M	18	10
```

## `uniq` (Deduplicate Rows)

The `uniq` command deduplicates rows of one or more TSV files without sorting. It uses a hash set to track unique keys.

### Basic Usage

```bash
tva uniq [input_files...] [options]
```

### Options

*   **`--fields` / `-f`**: TSV fields (1-based) to use as dedup key.
*   **`--header` / `-H`**: Treat the first line of each input as a header.
*   **`--ignore-case` / `-i`**: Ignore case when comparing keys.
*   **`--repeated` / `-r`**: Output only lines that are repeated based on the key.
*   **`--at-least` / `-a`**: Output only lines that are repeated at least INT times.
*   **`--max` / `-m`**: Max number of each unique key to output (zero is ignored).
*   **`--equiv` / `-e`**: Append equivalence class IDs rather than only uniq entries.
*   **`--number` / `-z`**: Append occurrence numbers for each key.

### Examples

#### 1. Deduplicate whole rows

```bash
tva uniq docs/data/us_rent_income.tsv --header
```

Output (first 5 lines):
```tsv
GEOID	NAME	variable	estimate	moe
01	Alabama	income	24476	136
01	Alabama	rent	747	3
02	Alaska	income	32940	508
```

#### 2. Deduplicate by a specific column

Deduplicate based on the `NAME` column:

```bash
tva uniq docs/data/us_rent_income.tsv --header -f NAME
```

Output (first 5 lines):
```tsv
GEOID	NAME	variable	estimate	moe
01	Alabama	income	24476	136
02	Alaska	income	32940	508
04	Arizona	income	27517	148
05	Arkansas	income	23789	165
```

#### 3. Output repeated lines only

Output lines where the `NAME` column appears more than once:

```bash
tva uniq docs/data/us_rent_income.tsv --header -f NAME --repeated
```

Output (first 5 lines):
```tsv
GEOID	NAME	variable	estimate	moe
01	Alabama	rent	747	3
02	Alaska	rent	1200	13
04	Arizona	rent	972	4
05	Arkansas	rent	709	5
```
