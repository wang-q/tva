# Reshaping Documentation

This document explains how to use the reshaping commands in `tva`: **`longer`** and **`wider`**. These commands are inspired by the `pivot_longer()` and `pivot_wider()` functions from the R package `tidyr`.

## Introduction

Data is often described as "long" or "wide":

*   **Long data**: Has more rows and fewer columns. This format is typically "tidy" and easier for analysis, where each row is an observation and each column is a variable.
*   **Wide data**: Has more columns and fewer rows. This format is often better for data entry or presentation (e.g., a spreadsheet with years as columns).

`tva` provides tools to switch between these formats:

*   **`longer`**: Reshapes "wide" data into a "long" format.
*   **`wider`**: Reshapes "long" data into a "wide" format.

**Comparison: `stats` vs `wider`**

| Feature | `stats` (Group By) | `wider` (Pivot) |
| :--- | :--- | :--- |
| **Goal** | Summarize to rows | Reshape to columns |
| **Output** | Long / Tall | Wide / Matrix |

## `longer` (Wide to Long)

The `longer` command is designed to reshape "wide" data into a "long" format. "Wide" data often has column names that are actually values of a variable. For example, a table might have columns like `2020`, `2021`, `2022` representing years. `longer` gathers these columns into a pair of key-value columns (e.g., `year` and `population`), making the data "longer" (more rows, fewer columns) and easier to analyze.

### Basic Usage

```bash
tva longer [input_files...] --cols <columns> [options]
```

*   **`--cols` / `-c`**: Specifies which columns to reshape. You can use column names, indices (1-based), or ranges (e.g., `3-5`, `wk*`).
*   **`--names-to`**: The name of the new column that will store the original column headers (default: "name").
*   **`--values-to`**: The name of the new column that will store the data values (default: "value").

## Examples

### 1. String Data in Column Names

Consider a dataset `docs/data/relig_income.tsv` where income brackets are spread across column names:

```tsv
religion	<$10k	$10-20k	$20-30k
Agnostic	27	34	60
Atheist	12	27	37
Buddhist	27	21	30
```

To tidy this, we want to turn the income columns into a single `income` variable:

```bash
tva longer docs/data/relig_income.tsv --cols 2-4 --names-to income --values-to count
```

Output:
```tsv
religion	income	count
Agnostic	<$10k	27
Agnostic	$10-20k	34
Agnostic	$20-30k	60
...
```

### 2. Numeric Data in Column Names

The `docs/data/billboard.tsv` dataset records song rankings by week (`wk1`, `wk2`, etc.):

```tsv
artist	track	wk1	wk2	wk3
2 Pac	Baby Don't Cry	87	82	72
2Ge+her	The Hardest Part	91	87	92
```

We can gather the week columns and strip the "wk" prefix to get a clean number:

```bash
tva longer docs/data/billboard.tsv --cols "wk*" --names-to week --values-to rank --names-prefix "wk" --values-drop-na
```

*   **`--names-prefix "wk"`**: Removes "wk" from the start of the column names (e.g., "wk1" -> "1").
*   **`--values-drop-na`**: Drops rows where the rank is missing (empty).

Output:
```tsv
artist	track	week	rank
2 Pac	Baby Don't Cry	1	87
2 Pac	Baby Don't Cry	2	82
...
```

### 3. Many Variables in Column Names (Regex Extraction)

Sometimes column names contain multiple pieces of information. For example, in the `docs/data/who.tsv` dataset, columns like `new_sp_m014` encode:
*   `new`: new cases (constant)
*   `sp`: diagnosis method
*   `m`: gender (m/f)
*   `014`: age group (0-14)

```tsv
country	iso2	iso3	year	new_sp_m014	new_sp_f014
Afghanistan	AF	AFG	1980	NA	NA
```

We can use **`--names-pattern`** with a regular expression to extract these parts into multiple columns:

```bash
tva longer docs/data/who.tsv --cols "new_*" --names-to diagnosis gender age --names-pattern "new_?(.*)_(.)(.*)"
```

*   **`--names-to`**: We provide 3 names for the 3 capture groups in the regex.
*   **`--names-pattern`**: The regex `new_?(.*)_(.)(.*)` captures:
    1.  `.*` (diagnosis, e.g., "sp")
    2.  `.` (gender, e.g., "m")
    3.  `.*` (age, e.g., "014")

Output:
```tsv
country	iso2	iso3	year	diagnosis	gender	age	value
Afghanistan	AF	AFG	1980	sp	m	014	NA
...
```

### 4. Splitting Column Names with a Separator

If column names are consistently separated by a character, you can use **`--names-sep`**.

Input `docs/data/household.tsv`:
```tsv
family	dob_child1	dob_child2	name_child1	name_child2
1	1998-11-26	2000-01-29	J	K
```

(Note: Handling "multiple observations per row" like `tidyr`'s `.value` sentinel is not yet fully supported in a single pass, but basic splitting is.)

For a simpler example, if columns are `year_semester` (e.g., `2020_1`, `2020_2`):

```bash
tva longer data.tsv --cols "20*" --names-to year semester --names-sep "_"
```

This splits `2020_1` into `2020` (year) and `1` (semester).

## `wider` (Long to Wide)

The `wider` command is the inverse of `longer`. It spreads a key-value pair across multiple columns, increasing the number of columns and decreasing the number of rows. This is useful for creating summary tables or reshaping data for tools that expect a matrix-like format.

### Basic Usage

```bash
tva wider [input_files...] --names-from <column> --values-from <column> [options]
```

*   **`--names-from`**: The column containing the new column names.
*   **`--values-from`**: The column containing the new column values.
*   **`--id-cols`**: (Optional) Columns that uniquely identify each row. If not specified, all columns except `names-from` and `values-from` are used.
*   **`--values-fill`**: (Optional) Value to use for missing cells (default: empty).
*   **`--names-sort`**: (Optional) Sort the new column headers alphabetically.
*   **`--op`**: (Optional) Aggregation operation (e.g., `sum`, `mean`, `count`, `last`). Default: `last`.

### Example 1: US Rent and Income

Consider the dataset `docs/data/us_rent_income.tsv`:

```tsv
GEOID	NAME	variable	estimate	moe
01	Alabama	income	24476	136
01	Alabama	rent	747	3
02	Alaska	income	32940	508
02	Alaska	rent	1200	13
```

Here, `variable` contains the type of measurement (`income` or `rent`), and `estimate` contains the value. To make this easier to compare, we can widen the data:

```bash
tva wider docs/data/us_rent_income.tsv --names-from variable --values-from estimate
```

Output:
```tsv
GEOID	NAME	income	rent
01	Alabama	24476	747
02	Alaska	32940	1200
...
```

**Understanding ID Columns**:
By default, `wider` uses all columns *except* `names-from` and `values-from` as ID columns. In this example, `GEOID`, `NAME`, and `moe` are treated as IDs.
However, `moe` (margin of error) is different for each row (it depends on the variable). If we include it as an ID, we might get multiple rows for the same state if we're not careful.
In this specific case, since `moe` is unique to the `variable`/`estimate` pair, `wider` handles it, but typically you might want to exclude such columns if they aren't part of the identifier.

To explicitly specify that only `GEOID` and `NAME` identify a row (and drop `moe`):

```bash
tva wider docs/data/us_rent_income.tsv --names-from variable --values-from estimate --id-cols GEOID,NAME
```

### Example 2: Capture-Recapture Data (Filling Missing Values)

The `docs/data/fish_encounters.tsv` dataset describes when fish were detected by monitoring stations. Some fish are seen at some stations but not others.

```tsv
fish	station	seen
4842	Release	1
4842	I80_1	1
4842	Lisbon	1
4843	Release	1
4843	I80_1	1
4844	Release	1
```

If we widen this by `station`, we will have missing values for stations where a fish wasn't seen. We can use **`--values-fill`** to fill these gaps with `0`.

```bash
tva wider docs/data/fish_encounters.tsv --names-from station --values-from seen --values-fill 0
```

Output:
```tsv
fish	Release	I80_1	Lisbon
4842	1	1	1
4843	1	1	0
4844	1	0	0
```

Without `--values-fill 0`, the missing cells would be empty strings (default).

## Complex Reshaping: Longer then Wider

Sometimes data requires multiple steps to be fully tidy. A common pattern is to make data longer to fix column headers, and then wider to separate variables.

Consider the `docs/data/world_bank_pop.tsv` dataset (a subset):

```tsv
country	indicator	2000	2001
ABW	SP.URB.TOTL	42444	43048
ABW	SP.URB.GROW	1.18	1.41
AFG	SP.URB.TOTL	4436311	4648139
AFG	SP.URB.GROW	3.91	4.66
```

Here, years are in columns (needs `longer`) and variables are in the `indicator` column (needs `wider`). We can pipe `tva` commands to solve this:

```bash
tva longer docs/data/world_bank_pop.tsv --cols 3-4 --names-to year --values-to value | \
tva wider --names-from indicator --values-from value
```

1.  **`longer`**: Reshapes years (cols 3-4) into `year` and `value`.
2.  **`wider`**: Takes the stream, uses `indicator` for new column names, and fills them with `value`. `country` and `year` automatically become ID columns.

Output:
```tsv
country	year	SP.URB.TOTL	SP.URB.GROW
ABW	2000	42444	1.18
ABW	2001	43048	1.41
AFG	2000	4436311	3.91
AFG	2001	4648139	4.66
```

## Handling Duplicates (Aggregation)

When widening data, you might encounter multiple rows for the same ID and name combination.

*   **`tidyr`**: Often creates list-columns or requires an aggregation function (`values_fn`).
*   **`tva`**: Supports aggregation via the **`--op`** argument.

By default (`--op last`), `tva` **overwrites** previous values with the **last observed value**.

However, you can specify an operation to aggregate these values, similar to `values_fn` in `tidyr` or `crosstab` in `datamash`.

Supported operations: `count`, `sum`, `mean`, `min`, `max`, `first`, `last`, `median`, `mode`, `stdev`, `variance`, etc.

### Example: Summing values

Example using `docs/data/warpbreaks.tsv`:
```tsv
wool	tension	breaks
A	L	26
A	L	30
A	L	54
...
```

If we want to sum the breaks for each wool/tension pair:
```bash
tva wider docs/data/warpbreaks.tsv --names-from wool --values-from breaks --op sum
```

Output:
```tsv
tension	A	B
L	110	54
M	87	63
H	72	84
```
(For A-L: 26 + 30 + 54 = 110)

### Example: Crosstab (Counting)

You can also use `wider` to create a frequency table (crosstab) by using `--op count`. In this case, `--values-from` is optional.

```bash
tva wider docs/data/warpbreaks.tsv --names-from wool --op count
```

Output:
```tsv
tension	A	B
L	3	3
M	3	3
H	3	3
```
(Each combination appears 3 times in this dataset)

### Comparison: `stats` vs `wider` (Aggregation)

Both `tva stats` (if available) and `tva wider --op ...` can aggregate data, but they produce different **structures**:

| Feature | `tva stats` (Group By) | `tva wider` (Pivot) |
| :--- | :--- | :--- |
| **Goal** | Summarize data into rows | Reshape data into columns |
| **Output Shape** | Long / Tall | Wide / Matrix |
| **Columns** | Fixed (Group + Stat) | Dynamic (Values become Headers) |
| **Best For** | General summaries, reporting | Cross-tabulation, heatmaps |

**Example**:
Data:
```tsv
Group   Category    Value
A       X           10
A       Y           20
B       X           30
B       Y           40
```

**`tva stats`** (Sum by Group):
```tsv
Group   Sum_Value
A       30
B       70
```
(Retains vertical structure)

**`tva wider`** (Sum, name from Category):
```tsv
Group   X   Y
A       10  20
B       30  40
```
(Spreads categories horizontally)

## Detailed Options

| Option | Description |
| :--- | :--- |
| `--cols <cols>` | **(Longer Only)** Columns to reshape. Supports indices (`1`, `1-3`), names (`year`), and wildcards (`wk*`). |
| `--names-to <names...>` | **(Longer Only)** Name(s) for the new key column(s). |
| `--values-to <name>` | **(Longer Only)** Name for the new value column. |
| `--names-from <col>` | **(Wider Only)** Column for new headers. |
| `--values-from <col>` | **(Wider Only)** Column for new values. |
| `--id-cols <cols>` | **(Wider Only)** Columns identifying rows. |
| `--values-fill <str>` | **(Wider Only)** Fill value for missing cells. |
| `--names-sort` | **(Wider Only)** Sort new column headers. |
| `--op <op>` | **(Wider Only)** Aggregation operation (`sum`, `mean`, `count`, etc.). |

## Comparison with R `tidyr`

| Feature | `tidyr::pivot_longer` | `tva longer` |
| :--- | :--- | :--- |
| Basic pivoting | `cols`, `names_to`, `values_to` | Supported |
| Drop NAs | `values_drop_na = TRUE` | `--values-drop-na` |
| Prefix removal | `names_prefix` | `--names-prefix` |
| Separator split | `names_sep` | `--names-sep` |
| Regex extraction | `names_pattern` | `--names-pattern` |

| Feature | `tidyr::pivot_wider` | `tva wider` |
| :--- | :--- | :--- |
| Basic pivoting | `names_from`, `values_from` | Supported |
| ID columns | `id_cols` (default: all others) | `--id-cols` (default: all others) |
| Fill missing | `values_fill` | `--values-fill` |
| Sort columns | `names_sort` | `--names-sort` |
| Aggregation | `values_fn` | `--op` (sum, mean, count, etc.) |
| Multiple values | `values_from = c(a, b)` | Not supported (single column only) |
| Multiple names | `names_from = c(a, b)` | Not supported (single column only) |
| Implicit missing | `names_expand`, `id_expand` | Not supported |

`tva` brings the power of tidy data reshaping to the command line, allowing for efficient processing of large TSV files without loading them entirely into memory.
