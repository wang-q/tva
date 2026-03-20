# from html

Extracts data from HTML files using CSS selectors.

Behavior:

This command converts HTML content into TSV format using three different modes:

1. **Query Mode**: For quick extraction of specific elements.
2. **Table Mode**: For automatically converting HTML tables (`<table>`).
3. **Struct Mode**: For extracting lists of objects into rows and columns.

**Input:**

* Reads from standard input if no input file is given or if the input file is 'stdin'.
* Supports plain text HTML files.

**Output:**

* Writes to standard output by default.
* Use `--outfile` / `-o` to write to a file (`[stdout]` for screen).

**Query Mode:**

* Activated by the `--query` / `-q` flag.
* Syntax: `selector [display_function]`
* **Selectors**: Standard CSS selectors (e.g., `div.content`, `#main a`).
* **Display Functions**:
    * `text{}` or `text()`: Print the text content of the selected elements.
    * `attr{name}` or `attr("name")`: Print the value of the specified attribute.
    * If omitted, prints the full HTML of selected elements.
* Empty results are kept by default (prints blank lines for empty text or missing attributes).
* For advanced CSS selector reference, see: `docs/selectors.md`.

**Table Mode:**

* Activated by the `--table` flag.
* Extracts data from HTML `<table>` elements.
* Use `--index N` to select the N-th matched table (1-based). Implies `--table`.
* Use `--table=<css>` to target specific tables (e.g., `div.result table`).

**Struct Mode (List Extraction):**

* Activated by using `--row` and `--col` flags.
* Designed to extract repetitive structures (like cards, list items) into a TSV table.
* `--row <selector>`: Defines the container for each record (e.g., `div.product`, `li`).
* `--col "Name:Selector [Function]"`: Defines a column in the output TSV.
    * `Name`: The column header name.
    * `Selector`: CSS selector relative to the row element.
    * `Function`: `text{}` (default) or `attr{name}`.
    * Example: `--col "Link:a.title attr{href}"`
    * Missing elements or attributes result in empty TSV cells.

Input:

* Reads from files or standard input.
* Use `stdin` or omit the file argument to read from standard input.

Output:

* By default, output is written to standard output.
* Use `--outfile` to write to a file instead.

Examples:

1. Extract all links (Query Mode)
   `tva from html -q "a attr{href}" index.html`

2. Extract the first table (Table Mode)
   `tva from html --table data.html`

3. Extract product list (Struct Mode)
   tva from html --row "div.product-card" \
       --col "Title: h2.title text{}" \
       --col "Price: .price" \
       --col "URL: a.link attr{href}" \
       products.html
