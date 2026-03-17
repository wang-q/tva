# Data Organization Documentation

This document explains how to use the data organization commands in `tva`: **`sort`**, **`reverse`**,
**`join`**, **`append`**, and **`split`**. These commands allow you to rearrange, combine, and split
your data.

## Introduction

Data organization involves sorting rows, combining multiple datasets, or splitting data into
multiple files. These operations are essential for data preparation and pipeline construction.

* **Sorting & Reversing**:
    * **`sort`**: Sorts rows based on one or more key fields.
    * **`reverse`**: Reverses the order of lines (like `tac`), optionally keeping the header at the top.
* **Combining**:
    * **`join`**: Joins two files based on common keys.
    * **`append`**: Concatenates multiple TSV files, handling headers correctly.
* **Splitting**:
    * **`split`**: Splits a file into multiple files (by size, key, or random).

## `sort` (External Sort)

The `sort` command sorts the lines of a TSV file based on the values in specified columns. It
supports both lexicographic (string) and numeric sorting.

### Basic Usage

```bash
tva sort [input_files...] [options]
```

* **`--key` / `-k`**: Specify the field(s) to use as the sort key. You can use 1-based indices (
  e.g., `1`, `2`) or ranges (e.g., `2,4-5`).
* **`--numeric` / `-n`**: Compare the key fields numerically instead of lexicographically.
* **`--reverse` / `-r`**: Reverse the sort result (descending order).

### Examples

#### 1. Sort by a single column (Lexicographic)

Sort `docs/data/us_rent_income.tsv` by the `NAME` column (column 2):

```bash
tva sort docs/data/us_rent_income.tsv -k 2
```

Output (first 5 lines):

```tsv
01	Alabama	income	24476	136
01	Alabama	rent	747	3
02	Alaska	income	32940	508
02	Alaska	rent	1200	13
04	Arizona	income	27517	148
```

#### 2. Sort numerically

Sort `docs/data/us_rent_income.tsv` by the `estimate` column (column 4) numerically:

```bash
tva sort docs/data/us_rent_income.tsv -k 4 -n
```

Output (first 5 lines):

```tsv
GEOID	NAME	variable	estimate	moe
05	Arkansas	rent	709	5
01	Alabama	rent	747	3
04	Arizona	rent	972	4
02	Alaska	rent	1200	13
```

#### 3. Sort by multiple columns

Sort first by `GEOID` (column 1), then by `NAME` (column 2):

```bash
tva sort docs/data/us_rent_income.tsv -k 1,2
```

## `reverse` (Reverse Lines)

The `reverse` command reverses the order of lines in the input. This is similar to the Unix `tac`
command but includes features specifically for tabular data, such as header preservation.

### Basic Usage

```bash
tva reverse [input_files...] [options]
```

* **`--header` / `-H`**: Treat the first line as a header and keep it at the top of the output.

### Examples

#### Reverse a file keeping the header

Reverse `docs/data/us_rent_income.tsv` but keep the header line at the top:

```bash
tva reverse docs/data/us_rent_income.tsv --header
```

Output (first 5 lines):

```tsv
GEOID	NAME	variable	estimate	moe
06	California	rent	1358	3
06	California	income	29454	109
05	Arkansas	rent	709	5
05	Arkansas	income	23789	165
```

## `join`

Joins lines from a TSV data stream against a filter file using one or more key fields.

### Examples

#### 1. Join two files by a common key

Using `docs/data/who.tsv` (contains `iso3`) and `docs/data/world_bank_pop.tsv` (contains `country`
with ISO3 codes):

```bash
tva join -H --filter-file docs/data/who.tsv --key-fields iso3 --data-fields country docs/data/world_bank_pop.tsv
```

Output:

```tsv
country	indicator	2000	2001
AFG	SP.URB.TOTL	4436311	4648139
AFG	SP.URB.GROW	3.91	4.66
```

#### 2. Append fields from the filter file

To add the `year` column from `who.tsv` to the output:

```bash
tva join -H --filter-file docs/data/who.tsv -k iso3 -d country --append-fields year docs/data/world_bank_pop.tsv
```

Output:

```tsv
country	indicator	2000	2001	year
AFG	SP.URB.TOTL	4436311	4648139	1980
AFG	SP.URB.GROW	3.91	4.66	1980
```

## `append`

Concatenates TSV files with optional header awareness and source tracking.

### Examples

#### 1. Concatenate files with headers

When appending multiple files with headers, use `-H` to keep only the header from the first file:

```bash
tva append -H docs/data/world_bank_pop.tsv docs/data/world_bank_pop.tsv
```

Output:

```tsv
country	indicator	2000	2001
ABW	SP.URB.TOTL	42444	43048
ABW	SP.URB.GROW	1.18	1.41
AFG	SP.URB.TOTL	4436311	4648139
AFG	SP.URB.GROW	3.91	4.66
ABW	SP.URB.TOTL	42444	43048
ABW	SP.URB.GROW	1.18	1.41
AFG	SP.URB.TOTL	4436311	4648139
AFG	SP.URB.GROW	3.91	4.66
```

#### 2. Track source file

Add a column indicating the source file:

```bash
tva append -H --track-source docs/data/world_bank_pop.tsv
```

Output:

```tsv
file	country	indicator	2000	2001
world_bank_pop	ABW	SP.URB.TOTL	42444	43048
world_bank_pop	ABW	SP.URB.GROW	1.18	1.41
...
```

## `split`

Splits TSV rows into multiple output files.

### Usage

Split `file.tsv` into multiple files with 1000 lines each:

```bash
tva split --lines-per-file 1000 --header-in-out file.tsv
```

This will create files like `file_0001.tsv`, `file_0002.tsv`, etc., each containing up to 1000 data
rows (plus the header in each file if `--header-in-out` is used).
