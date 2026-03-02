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
*   **`html`**: Extract data from HTML to TSV.
    *   `tva from html [input] [-q query] [--table] [--row selector --col spec] ...`

### Examples

Convert a CSV file to TSV:
```bash
tva from csv docs/data/input.csv > output.tsv
```

Convert an Excel sheet to TSV:
```bash
tva from xlsx docs/data/formats.xlsx --sheet "Introduction" > output.tsv
```

Extract all links from an HTML file:
```bash
tva from html -q "nav a attr{href}" docs/data/sample.html
```

Convert an HTML table to TSV:
```bash
tva from html --table=".specs-table" docs/data/sample.html
```

Extract structured data (rows and columns) from HTML:
```bash
tva from html --row ".product-card" \
    --col "Name:.title" \
    --col "Price:.price" \
    --col "Link:a.buy-btn attr{href}" \
    docs/data/sample.html
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
