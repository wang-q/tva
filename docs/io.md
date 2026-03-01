# Import & Export Documentation

This document explains how to use the import/export commands in `tva`: **`from`** and **`to`**.

## `from`

Converts other formats (CSV, XLSX) to TSV.

### Usage

```bash
tva from <SUBCOMMAND> [options]
```

### Subcommands

*   **`csv`**: Convert CSV to TSV.
    *   `tva from csv [input] [-o output] [-d delimiter]`
*   **`xlsx`**: Convert XLSX to TSV.
    *   `tva from xlsx [input] [--sheet name] [--list-sheets]`

### Examples

Convert a CSV file to TSV:
```bash
tva from csv docs/data/input.csv > output.tsv
```

Convert an Excel sheet to TSV:
```bash
tva from xlsx docs/data/formats.xlsx --sheet "Introduction" > output.tsv
```

## `to`

Converts TSV to other formats (CSV, XLSX).

### Usage

```bash
tva to <SUBCOMMAND> [options]
```

### Subcommands

*   **`csv`**: Convert TSV to CSV.
    *   `tva to csv [input] [-o output] [-d delimiter]`
*   **`xlsx`**: Convert TSV to XLSX.
    *   `tva to xlsx [input] [-o output.xlsx] [-H] [--le col:val] ...`

### Examples

Convert a TSV file to CSV:
```bash
tva to csv docs/data/household.tsv > output.csv
```

Convert a TSV file to XLSX with formatting:
```bash
tva to xlsx docs/data/household.tsv -o output.xlsx -H --le 1:2
```

Convert a TSV file to XLSX with multiple formatting rules:
```bash
tva to xlsx docs/data/rocauc.result.tsv -o output.xlsx \
    -H --le 4:0.5 --ge 4:0.6 --bt 4:0.52:0.58 --str-in-fld 1:m03
```

![to xlsx output](data/to_xlsx.png)
