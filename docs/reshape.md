# Reshaping Documentation

This document explains how to use the reshaping commands in `tva`: **`longer`** and (planned) **`wider`**. These commands are inspired by the `pivot_longer()` and `pivot_wider()` functions from the R package `tidyr`.

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

Consider a dataset `relig_income.tsv` where income brackets are spread across column names:

```tsv
religion	<$10k	$10-20k	$20-30k
Agnostic	27	34	60
Atheist	12	27	37
Buddhist	27	21	30
```

To tidy this, we want to turn the income columns into a single `income` variable:

```bash
tva longer relig_income.tsv --cols 2-4 --names-to income --values-to count
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

The `billboard.tsv` dataset records song rankings by week (`wk1`, `wk2`, etc.):

```tsv
artist	track	wk1	wk2	wk3
2 Pac	Baby Don't Cry	87	82	72
2Ge+her	The Hardest Part	91	87	92
```

We can gather the week columns and strip the "wk" prefix to get a clean number:

```bash
tva longer billboard.tsv --cols "wk*" --names-to week --values-to rank --names-prefix "wk" --values-drop-na
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

Sometimes column names contain multiple pieces of information. For example, in the `who.tsv` dataset, columns like `new_sp_m014` encode:
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
tva longer who.tsv --cols "new_*" --names-to diagnosis gender age --names-pattern "new_?(.*)_(.)(.*)"
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

Input `household.tsv`:
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

## Detailed Options

| Option | Description |
| :--- | :--- |
| `--cols <cols>` | **Required.** Columns to include in the reshape. Supports indices (`1`, `1-3`), names (`year`), and wildcards (`wk*`). |
| `--names-to <names...>` | Name(s) for the new key column(s). Default: `name`. |
| `--values-to <name>` | Name for the new value column. Default: `value`. |
| `--values-drop-na` | If set, omits rows where the value cell is empty. |
| `--names-prefix <str>` | Removes the specified string from the beginning of reshaped column names. |
| `--names-sep <char>` | Splits column names into multiple columns using a separator character. |
| `--names-pattern <regex>` | Uses a regular expression with capture groups to extract multiple columns from column names. |

## Comparison with R `tidyr::pivot_longer`

| Feature | `tidyr::pivot_longer` | `tva longer` |
| :--- | :--- | :--- |
| Basic pivoting | `cols`, `names_to`, `values_to` | Supported |
| Drop NAs | `values_drop_na = TRUE` | `--values-drop-na` |
| Prefix removal | `names_prefix` | `--names-prefix` |
| Separator split | `names_sep` | `--names-sep` |
| Regex extraction | `names_pattern` | `--names-pattern` |
| Type conversion | `names_transform` | Not supported (use separate tool) |
| Multiple values | `.value` sentinel | Not directly supported |

`tva longer` brings the power of tidy data reshaping to the command line, allowing for efficient processing of large TSV files without loading them entirely into memory.
