# Selection & Filtering Documentation

This document explains how to use the selection, filtering, and sampling commands in `tva`:
**`select`**, **`filter`**, **`slice`**, and **`sample`**. These commands allow you to subset your
data based on structure (columns), values (rows), position (index), or randomly.

## Introduction

Data analysis often begins with selecting the relevant subset of data:

* **`select`**: Selects and reorders columns (e.g., "keep only `name` and `email`").
* **`filter`**: Selects rows where a condition is true (e.g., "keep rows where `age > 30`").
* **`slice`**: Selects rows by their position (index) in the file (e.g., "keep rows 10-20").
* **`sample`**: Randomly selects a subset of rows.

## Field Syntax

All tools use a unified syntax to identify fields (columns).
See [Field Syntax Documentation](help/fields.md) for details.

## `select` (Column Selection)

The `select` command allows you to keep only specific columns and reorder them.

### Basic Usage

```bash
tva select [input_files...] --fields <columns>
```

* **`--fields` / `-f`**: Comma-separated list of columns to select.
    * **Names**: `name`, `email`
    * **Indices**: `1`, `3` (1-based)
    * **Ranges**: `1-3`, `start_col-end_col`
    * **Wildcards**: `user_*`, `*_id`

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
tva select docs/data/us_rent_income.tsv -f NAME,estimate
```

Output:

```tsv
NAME	estimate
Alabama	24476
Alabama	747
Alaska	32940
...
```

#### 2. Reorder Columns

You can change the order of columns. Let's move `variable` to the first column:

```bash
tva select docs/data/us_rent_income.tsv -f variable,estimate,NAME
```

Output:

```tsv
variable	estimate	NAME
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
tva select docs/data/billboard.tsv -f artist,track,wk*
```

Or using a range (if you know the indices):

```bash
tva select docs/data/billboard.tsv -f 1-2,3-5
```

## `filter` (Row Filtering)

The `filter` command selects rows where a condition is true. It supports field-based tests,
expressions, empty/blank checks, and field-to-field comparisons.

### Basic Usage

```bash
tva filter [input_files...] [options]
```

Filter tests can be combined (default is AND logic, use `--or` for OR logic).

### Filter Types

#### 1. Expression Filter

Use the `-E` option to filter with an expression:

```bash
tva filter docs/data/us_rent_income.tsv -H -E '@estimate > 30000'
```

#### 2. Empty/Blank Checks

* `--empty <field>`: True if the field is empty (no characters)
* `--not-empty <field>`: True if the field is not empty
* `--blank <field>`: True if the field is empty or all whitespace
* `--not-blank <field>`: True if the field contains a non-whitespace character

```bash
tva filter docs/data/us_rent_income.tsv --not-empty NAME
```

#### 3. Numeric Comparison

Format: `--<op> <field>:<value>`

* `--eq`, `--ne`, `--gt`, `--ge`, `--lt`, `--le`

```bash
tva filter docs/data/us_rent_income.tsv --gt estimate:30000
```

Output:

```tsv
GEOID	NAME	variable	estimate	moe
02	Alaska	income	32940	508
04	Arizona	income	31614	242
06	California	income	33095	172
...
```

#### 4. String Comparison

* `--str-eq`, `--str-ne`: String equality/inequality
* `--str-gt`, `--str-ge`, `--str-lt`, `--str-le`: String ordering
* `--istr-eq`, `--istr-ne`: Case-insensitive string comparison
* `--str-in-fld`, `--str-not-in-fld`: Substring test
* `--istr-in-fld`, `--istr-not-in-fld`: Case-insensitive substring test

```bash
tva filter docs/data/us_rent_income.tsv --str-eq variable:rent
```

Output:

```tsv
GEOID	NAME	variable	estimate	moe
01	Alabama	rent	747	3
02	Alaska	rent	1200	13
04	Arizona	rent	976	4
...
```

#### 5. Regular Expression

* `--regex <field>:<pattern>`: Field matches regex
* `--iregex <field>:<pattern>`: Case-insensitive regex match
* `--not-regex <field>:<pattern>`: Field does not match regex
* `--not-iregex <field>:<pattern>`: Case-insensitive non-match

```bash
tva filter docs/data/billboard.tsv --regex track:"Baby"
```

Output:

```tsv
artist	track	wk1	wk2	wk3
2 Pac	Baby Don't Cry	87	82	72
Beenie Man	Girls Dem Sugar	87	70	63
...
```

#### 6. Length Comparison

* `--char-len-eq`, `--char-len-ne`, `--char-len-gt`, `--char-len-ge`, `--char-len-lt`,
  `--char-len-le`: Character length
* `--byte-len-eq`, `--byte-len-ne`, `--byte-len-gt`, `--byte-len-ge`, `--byte-len-lt`,
  `--byte-len-le`: Byte length

```bash
tva filter docs/data/billboard.tsv --char-len-gt track:10
```

#### 7. Field Type Checks

* `--is-numeric <field>`: True if field can be parsed as a number
* `--is-finite <field>`: True if field is numeric and finite
* `--is-nan <field>`: True if field is NaN
* `--is-infinity <field>`: True if field is positive or negative infinity

```bash
tva filter docs/data/us_rent_income.tsv --is-numeric estimate
```

#### 8. Field-to-Field Comparison

* `--ff-eq`, `--ff-ne`, `--ff-lt`, `--ff-le`, `--ff-gt`, `--ff-ge`: Numeric field-to-field
* `--ff-str-eq`, `--ff-str-ne`: String field-to-field
* `--ff-istr-eq`, `--ff-istr-ne`: Case-insensitive string field-to-field
* `--ff-absdiff-le <f1>:<f2>:<num>`: Absolute difference <= NUM
* `--ff-absdiff-gt <f1>:<f2>:<num>`: Absolute difference > NUM
* `--ff-reldiff-le <f1>:<f2>:<num>`: Relative difference <= NUM
* `--ff-reldiff-gt <f1>:<f2>:<num>`: Relative difference > NUM

```bash
tva filter docs/data/us_rent_income.tsv --ff-gt estimate:moe
```

### Common Options

* `--or`: Evaluate tests as OR instead of AND
* `-v`, `--invert`: Invert the filter, selecting non-matching rows
* `-c`, `--count`: Print only the count of matching data rows
* `--label <header>`: Label matched records instead of filtering (outputs 1/0)
* `--label-values <pass:fail>`: Custom values for --label (default: 1:0)

## `slice` (Row Selection by Index)

The `slice` command selects rows based on their integer index (position). Indices are 1-based.

### Basic Usage

```bash
tva slice [input_files...] --rows <range> [options]
```

* **`--rows` / `-r`**: The range of rows to keep (e.g., `1-10`, `5`, `100-`). Can be specified
  multiple times.
* **`--invert` / `-v`**: Invert selection (drop the specified rows).
* **`--header` / `-H`**: Always preserve the first row (header).

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

* **`--rate` / `-r`**: Sampling rate (probability 0.0-1.0). (Bernoulli sampling)
* **`--n` / `-n`**: Exact number of rows to sample. (Reservoir sampling)
* **`--seed` / `-s`**: Random seed for reproducibility.

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
