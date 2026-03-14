# `from` Command Documentation

The `from` command converts other file formats (CSV, XLSX, HTML) into TSV (Tab-Separated Values).

## Usage

```bash
tva from <SUBCOMMAND> [options]
```

## Subcommands

* **`csv`**: Convert CSV to TSV.
* **`xlsx`**: Convert XLSX to TSV.
* **`html`**: Extract data from HTML to TSV.

---

## `tva from csv`

Converts Comma-Separated Values (CSV) files to TSV. It handles standard CSV escaping, quoting, and
different delimiters.

### Usage

```bash
tva from csv [input] [options]
```

### Options

* `-o <file>` / `--outfile <file>`: Output filename (default: stdout).
* `-d <char>` / `--delimiter <char>`: Specify the input delimiter (default: `,`).

### Examples

**Convert a standard CSV file:**

```bash
tva from csv docs/data/input.csv
```

Output:

```tsv
Type    Value1  Value2
Vanilla ABC     123
Quoted  ABC     123
...
```

**Convert a semicolon-separated file:**

```bash
# Assuming input.csv uses ';'
tva from csv input.csv -d ";"
```

---

## `tva from xlsx`

Converts Excel (XLSX) spreadsheets to TSV.

### Usage

```bash
tva from xlsx [input] [options]
```

### Options

* `-o <file>` / `--outfile <file>`: Output filename (default: stdout).
* `--sheet <name>`: Select a specific sheet by name (default: first sheet).
* `--list-sheets`: List all sheet names in the file and exit.

### Examples

**List sheets in an Excel file:**

```bash
tva from xlsx docs/data/formats.xlsx --list-sheets
```

Output:

```
1: Introduction
2: Fonts
3: Named colors
...
```

**Extract a specific sheet:**

```bash
tva from xlsx docs/data/formats.xlsx --sheet "Introduction"
```

Output:

```tsv
This workbook demonstrates some of
the formatting options provided by
...
```

---

## `tva from html`

Extracts data from HTML files using CSS selectors. It supports three modes:

1. **Query Mode**: Extract specific elements (like `pup`).
2. **Table Mode**: Automatically extract HTML tables to TSV.
3. **List Mode**: Extract structured lists (e.g., product cards, news items) to TSV.

For a complete CSS selector reference, see [CSS Selectors](selectors.md).

### Usage

```bash
tva from html [input] [options]
```

### Options

* `-o <file>` / `--outfile <file>`: Output filename (default: stdout).
* `-q <query>` / `--query <query>`: Selector + optional display function (e.g., `a attr{href}`).
* `--table [selector]`: Extract standard HTML tables.
* `--index <N>`: Select the N-th table (1-based). Implies `--table`.
* `--row <selector>`: Selector for rows (List Mode).
* `--col <name:selector func>`: Column definition (List Mode). Can be used multiple times.

### Examples

**Query Mode: Extract all links**

```bash
tva from html -q "a attr{href}" docs/data/sample.html
```

**Table Mode: Extract the first table**

```bash
tva from html --table docs/data/sample.html
```

**Table Mode: Extract a specific table by class**

```bash
tva from html --table=".specs-table" docs/data/sample.html
```

Output:

```tsv
Feature Value
Weight  1.2 kg
Color   Silver
Warranty        2 Years
```

**List Mode: Extract structured product data**

```bash
tva from html --row ".product-card" \
    --col "Name:.title" \
    --col "Price:.price" \
    --col "Link:a.buy-btn attr{href}" \
    docs/data/sample.html
```

Output:

```tsv
Name    Price   Link
Super Widget    $19.99  /buy/widget
Mega Gadget     $29.99  /buy/gadget
```
