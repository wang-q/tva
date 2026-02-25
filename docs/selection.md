# Selection & Sampling Documentation

This document explains how to use the selection and sampling commands in `tva`: **`select`**, **`slice`**, and **`sample`**. These commands allow you to subset your data based on structure (columns) or position (rows).

## Introduction

Data analysis often begins with selecting the relevant subset of data:

*   **`select`**: Selects, reorders, and renames columns (e.g., "keep only `name` and `email`").
*   **`slice`**: Selects rows by their position (index) in the file (e.g., "keep rows 10-20").
*   **`sample`**: Randomly selects a subset of rows.

## Field Syntax

All tools use a unified syntax to identify fields (columns). See [Field Syntax Documentation](help/fields.md) for details.

## `select` (Column Selection)

The `select` command allows you to keep only specific columns, reorder them, and rename them.

### Basic Usage

```bash
tva select [input_files...] --cols <columns>
```

*   **`--cols` / `-c`**: Comma-separated list of columns to select.
    *   **Names**: `name`, `email`
    *   **Indices**: `1`, `3` (1-based)
    *   **Ranges**: `1-3`, `start_col-end_col`
    *   **Wildcards**: `user_*`, `*_id`
    *   **Exclusion**: `!password` (exclude column)
    *   **Renaming**: `new_name:old_name`

### Examples

#### 1. Select by Name and Index

Consider the dataset `docs/data/us_rent_income.tsv`:

```tsv
GEOID	NAME	variable	estimate	moe
01	Alabama	income	24476	136
01	Alabama	rent	747	3
02	Alaska	income	32940	508
...
```

To keep only the state name (`NAME`) and the estimate value (`estimate`):

```bash
tva select docs/data/us_rent_income.tsv -c NAME,estimate
```

Output:
```tsv
NAME	estimate
Alabama	24476
Alabama	747
Alaska	32940
...
```

#### 2. Reorder and Rename Columns

You can change the order of columns and rename them in a single step. Let's move `variable` to the first column and rename `estimate` to `Value`:

```bash
tva select docs/data/us_rent_income.tsv -c variable,Value:estimate,State:NAME
```

Output:
```tsv
variable	Value	State
income	24476	Alabama
rent	747	Alabama
income	32940	Alaska
...
```

#### 3. Select by Range and Wildcard

Consider `docs/data/billboard.tsv` which has many week columns (`wk1`, `wk2`, `wk3`...):

```tsv
artist	track	wk1	wk2	wk3
2 Pac	Baby Don't Cry	87	82	72
2Ge+her	The Hardest Part	91	87	92
```

To select the artist, track, and all week columns:

```bash
tva select docs/data/billboard.tsv -c artist,track,wk*
```

Or using a range (if you know the indices):

```bash
tva select docs/data/billboard.tsv -c 1-2,3-5
```

## `slice` (Row Selection by Index)

The `slice` command selects rows based on their integer index (position). Indices are 1-based.

### Basic Usage

```bash
tva slice [input_files...] --rows <range> [options]
```

*   **`--rows` / `-r`**: The range of rows to keep (e.g., `1-10`, `5`, `100-`). Can be specified multiple times.
*   **`--invert` / `-v`**: Invert selection (drop the specified rows).
*   **`--header` / `-H`**: Always preserve the first row (header).

### Examples

#### 1. Keep Specific Range (Head/Body)

To inspect the first 5 rows of `docs/data/billboard.tsv` (including header):

```bash
tva slice docs/data/billboard.tsv -r 1-5
```

Output:
```tsv
artist	track	wk1	wk2	wk3
2 Pac	Baby Don't Cry	87	82	72
2Ge+her	The Hardest Part	91	87	92
...
```

#### 2. Drop Header (Data Only)

Sometimes you want to process data without the header. You can drop the first row using `--invert`:

```bash
tva slice docs/data/billboard.tsv -r 1 --invert
```

Output:
```tsv
2 Pac	Baby Don't Cry	87	82	72
2Ge+her	The Hardest Part	91	87	92
...
```

#### 3. Keep Header and Specific Data Rows

To keep the header (row 1) and a slice of data from the middle (rows 10-15), use the `-H` flag:

```bash
tva slice docs/data/us_rent_income.tsv -H -r 10-15
```

This ensures the first line is always printed, even if it's not in the range `10-15`.

## `sample` (Random Sampling)

The `sample` command randomly selects a subset of rows. This is useful for exploring large datasets.

### Basic Usage

```bash
tva sample [input_files...] [options]
```

*   **`--rate` / `-r`**: Sampling rate (probability 0.0-1.0). (Bernoulli sampling)
*   **`--n` / `-n`**: Exact number of rows to sample. (Reservoir sampling)
*   **`--seed` / `-s`**: Random seed for reproducibility.

### Examples

#### 1. Sample by Rate

To keep approximately 10% of the rows from `docs/data/us_rent_income.tsv`:

```bash
tva sample docs/data/us_rent_income.tsv -r 0.1
```

#### 2. Sample Exact Number

To pick exactly 5 random rows for inspection:

```bash
tva sample docs/data/us_rent_income.tsv -n 5
```

Output (example):
```tsv
GEOID	NAME	variable	estimate	moe
35	New Mexico	rent	809	11
55	Wisconsin	income	32018	247
18	Indiana	rent	782	5
...
```
