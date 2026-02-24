# Ordering Documentation

This document explains how to use the ordering commands in `tva`: **`sort`**, **`reverse`**, and **`transpose`**. These commands allow you to rearrange the rows and columns of your data.

## Introduction

Ordering is a crucial step in data preparation and analysis. You might need to sort data to find the top items, reverse the order to see the most recent entries first, or transpose a matrix to swap rows and columns.

*   **`sort`**: Sorts rows based on one or more key fields.
*   **`reverse`**: Reverses the order of lines (like `tac`), optionally keeping the header at the top.
*   **`transpose`**: Swaps rows and columns (matrix transposition).

## `sort` (External Sort)

The `sort` command sorts the lines of a TSV file based on the values in specified columns. It supports both lexicographic (string) and numeric sorting.

### Basic Usage

```bash
tva sort [input_files...] [options]
```

*   **`--key` / `-k`**: Specify the field(s) to use as the sort key. You can use 1-based indices (e.g., `1`, `2`) or ranges (e.g., `2,4-5`).
*   **`--numeric` / `-n`**: Compare the key fields numerically instead of lexicographically.
*   **`--reverse` / `-r`**: Reverse the sort result (descending order).

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

The `reverse` command reverses the order of lines in the input. This is similar to the Unix `tac` command but includes features specifically for tabular data, such as header preservation.

### Basic Usage

```bash
tva reverse [input_files...] [options]
```

*   **`--header` / `-H`**: Treat the first line as a header and keep it at the top of the output.

### Examples

#### 1. Reverse a file keeping the header

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

## `transpose` (Matrix Transpose)

The `transpose` command swaps the rows and columns of a TSV file. It reads the entire file into memory and performs a matrix transposition.

### Basic Usage

```bash
tva transpose [input_file] [options]
```

### Notes

*   **Strict Mode**: `transpose` expects a rectangular matrix. All rows must have the same number of columns as the first row. If the file is jagged (rows have different lengths), the command will fail with an error.
*   **Memory Usage**: Since it reads the whole file, be cautious with very large files.

### Examples

#### 1. Transpose a table

Transpose `docs/data/relig_income.tsv`:

```bash
tva transpose docs/data/relig_income.tsv
```

Output (first 5 lines):
```tsv
religion	Agnostic	Atheist	Buddhist
<$10k	27	12	27
$10-20k	34	27	21
$20-30k	60	37	30
$30-40k	81	25	34
```
