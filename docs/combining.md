# Combining & Splitting Documentation

This document explains how to use the combining and splitting commands in `tva`: **`join`**, **`append`**, and **`split`**.

## `join`

Joins lines from a TSV data stream against a filter file using one or more key fields.

### Examples

#### 1. Join two files by a common key

Using `docs/data/who.tsv` (contains `iso3`) and `docs/data/world_bank_pop.tsv` (contains `country` with ISO3 codes):

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

Split `file.tsv` into 5 files randomly:

```bash
tva split --num-files 5 --header-in-out file.tsv
```

Split by key (rows with same key go to same file):

```bash
tva split --num-files 5 --key-fields 1 --header-in-out file.tsv
```
