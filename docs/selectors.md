# CSS Selectors Reference

`tva from html` uses the `scraper` crate which implements a robust subset of CSS selectors. This
document provides a comprehensive reference and examples, inspired by `pup`.

## Basic Selectors

| Selector | Description                             | Example    | Matches                 |
|:---------|:----------------------------------------|:-----------|:------------------------|
| `tag`    | Selects elements by tag name.           | `div`      | `<div>...</div>`        |
| `.class` | Selects elements by class.              | `.content` | `<div class="content">` |
| `#id`    | Selects elements by ID.                 | `#header`  | `<div id="header">`     |
| `*`      | Universal selector, matches everything. | `*`        | Any element             |

## Combinators

Combinators allow you to select elements based on their relationship to other elements.

| Selector | Name             | Description                      | Example                                |
|:---------|:-----------------|:---------------------------------|:---------------------------------------|
| `A B`    | Descendant       | Selects B inside A (any depth).  | `div p` (paragraphs inside divs)       |
| `A > B`  | Child            | Selects B directly inside A.     | `ul > li` (direct children list items) |
| `A + B`  | Adjacent Sibling | Selects B immediately after A.   | `h1 + p` (paragraph right after h1)    |
| `A ~ B`  | General Sibling  | Selects B after A (same parent). | `h1 ~ p` (all paragraphs after h1)     |
| `A, B`   | Grouping         | Selects both A and B.            | `h1, h2` (all h1 and h2 headers)       |

## Attribute Selectors

Filter elements based on their attributes.

| Selector        | Description                                      | Example                                         |
|:----------------|:-------------------------------------------------|:------------------------------------------------|
| `[attr]`        | Has attribute `attr`.                            | `[href]`                                        |
| `[attr="val"]`  | Attribute exactly equals `val`.                  | `[type="text"]`                                 |
| `[attr~="val"]` | Attribute contains word `val` (space separated). | `[class~="btn"]`                                |
| `[attr          | ="val"]`                                         | Attribute starts with `val` (hyphen separated). | `[lang|="en"]` |
| `[attr^="val"]` | Attribute starts with `val`.                     | `[href^="https"]`                               |
| `[attr$="val"]` | Attribute ends with `val`.                       | `[href$=".pdf"]`                                |
| `[attr*="val"]` | Attribute contains substring `val`.              | `[href*="google"]`                              |

## Pseudo-classes

Pseudo-classes select elements based on their state or position in the document tree.

### Structural & Position

| Selector               | Description                                 | Example                  |
|:-----------------------|:--------------------------------------------|:-------------------------|
| `:first-child`         | First child of its parent.                  | `li:first-child`         |
| `:last-child`          | Last child of its parent.                   | `li:last-child`          |
| `:only-child`          | Elements that are the only child.           | `p:only-child`           |
| `:first-of-type`       | First element of its type among siblings.   | `p:first-of-type`        |
| `:last-of-type`        | Last element of its type among siblings.    | `p:last-of-type`         |
| `:only-of-type`        | Only element of its type among siblings.    | `img:only-of-type`       |
| `:nth-child(n)`        | Selects the n-th child (1-based).           | `tr:nth-child(2)`        |
| `:nth-last-child(n)`   | n-th child from end.                        | `li:nth-last-child(1)`   |
| `:nth-of-type(n)`      | n-th element of its type.                   | `p:nth-of-type(2)`       |
| `:nth-last-of-type(n)` | n-th element of its type from end.          | `tr:nth-last-of-type(2)` |
| `:empty`               | Elements with no children (including text). | `td:empty`               |

**Note on `nth-child` arguments:**

* `2`: The 2nd child.
* `odd`: 1st, 3rd, 5th...
* `even`: 2nd, 4th, 6th...
* `2n+1`: Every 2nd child starting from 1 (1, 3, 5...).
* `3n`: Every 3rd child (3, 6, 9...).

### Logic & Content

| Selector            | Description                                                  | Example                                 |
|:--------------------|:-------------------------------------------------------------|:----------------------------------------|
| `:not(selector)`    | Elements that do NOT match the selector.                     | `input:not([type="submit"])`            |
| `:is(selector)`     | Matches any of the selectors in the list.                    | `:is(header, footer) a`                 |
| `:where(selector)`  | Same as `:is` but with 0 specificity.                        | `:where(section, article)`              |
| `:has(selector)`    | **(Experimental)** Elements containing specific descendants. | `div:has(img)`                          |
| `:contains("text")` | Not supported by `scraper`.                                  | *(Use `text{}` and filter downstream.)* |

## Display Functions

When using `tva from html -q`, you can append a display function to format the output. If omitted,
the full HTML of selected elements is printed.

| Function     | Description                                         | Example Output        |
|:-------------|:----------------------------------------------------|:----------------------|
| `text{}`     | Prints text content of element and children.        | `Hello World`         |
| `attr{name}` | Prints value of attribute `name`.                   | `https://example.com` |
| `json{}`     | **(Not yet implemented)** Output as JSON structure. | *N/A*                 |

> Note: `pup` supports `json{}`, but `tva` currently focuses on TSV/Text extraction. Use
`Struct Mode` (`--row`/`--col`) for structured data extraction.

## Known Limitations

The following features from `pup` are **not planned** for implementation:

* `json{}` output mode (use `text{}` or `attr{}` with TSV output).
* `pup`-specific pseudo-classes (e.g., `:parent-of`).
* `:contains()` selector (not supported by the underlying `scraper` engine).

## Examples

### Basic Filtering

Extract page title:

```bash
tva from html -q "title text{}" index.html
```

Extract all links from a specific list:

```bash
tva from html -q "ul#menu > li > a attr{href}" index.html
```

### Advanced Filtering

Extract rows from the second table on the page, skipping the header:

```bash
tva from html -q "table:nth-of-type(2) tr:nth-child(n+2)" index.html
```

Find all images that are NOT icons:

```bash
tva from html -q "img:not(.icon) attr{src}" index.html
```

Extract meta description:

```bash
tva from html -q "meta[name='description'] attr{content}" index.html
```
