# Row Filtering Documentation

This document explains how to use the **`filter`** command in `tva`. This command allows you to subset your data based on row values.

## `filter` (Row Filtering)

The `filter` command selects rows where a condition is true. It supports numeric, string, regular expression, and date comparisons.

### Basic Usage

```bash
tva filter [input_files...] [options] <criteria>
```

*   **`<criteria>`**: The condition to check. Format: `<column><operator><value>`.
    *   **Operators**:
        *   **Numeric**: `==`, `!=`, `>`, `>=`, `<`, `<=`
        *   **String**: `str-eq`, `str-ne`, `str-contains`, `str-starts-with`, `str-ends-with`
        *   **Regex**: `regex-match`, `regex-not-match`
        *   **Date**: `date-eq`, `date-ne`, `date-gt`, `date-ge`, `date-lt`, `date-le`

### Examples

#### 1. Numeric Filtering

Filter rows where the value in column `estimate` is greater than 30,000 in `docs/data/us_rent_income.tsv`:

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

#### 2. String Filtering

Filter rows where the `variable` column equals "rent" in `docs/data/us_rent_income.tsv`:

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

#### 3. Regex Filtering

Filter rows where the `track` column contains "Baby" in `docs/data/billboard.tsv`:

```bash
tva filter docs/data/billboard.tsv --regex-match "track:Baby"
```

Output:
```tsv
artist	track	wk1	wk2	wk3
2 Pac	Baby Don't Cry	87	82	72
Beenie Man	Girls Dem Sugar	87	70	63
...
```

#### 4. Date Filtering

Filter rows where `dob_child1` is after 1999-01-01 in `docs/data/household.tsv`:

```bash
tva filter docs/data/household.tsv --date-gt dob_child1:1999-01-01
```

(Note: Ensure your date format is ISO 8601 YYYY-MM-DD)
