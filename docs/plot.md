# Plotting Documentation

This document explains how to use the plotting commands in `tva`: **`plot point`**. These commands bring data visualization capabilities to the terminal, inspired by the grammar of graphics philosophy of `ggplot2`.

## Introduction

Terminal-based plotting allows you to quickly visualize data without leaving the command line. `tva` provides plotting tools that render directly in your terminal using ASCII/Unicode characters:

*   **`plot point`**: Draws scatter plots or line charts from TSV data.

## `plot point` (Scatter Plots and Line Charts)

The `plot point` command creates scatter plots or line charts directly in your terminal. It maps TSV columns to visual aesthetics (position, color) and renders the chart using ASCII/Unicode characters.

### Basic Usage

```bash
tva plot point [input_file] --x <column> --y <column> [options]
```

*   **`-x` / `--x`**: The column for X-axis position (required).
*   **`-y` / `--y`**: The column for Y-axis position (required).
*   **`--color`**: Column for grouping/coloring points by category (optional).
*   **`-l` / `--line`**: Draw line chart instead of scatter plot.

### Column Specification

Columns can be specified by:
*   **Header name**: e.g., `-x age`, `-y income`
*   **1-based index**: e.g., `-x 1`, `-y 3`

## Examples

### 1. Basic Scatter Plot

The simplest use case is plotting two numeric columns against each other.

Using the `tests/data/plot/iris.tsv` dataset (Fisher's Iris dataset):

```bash
tva plot point tests/data/plot/iris.tsv -x sepal_length -y sepal_width --cols 80 --rows 24
```

This creates a scatter plot showing the relationship between sepal length and sepal width.

Output (terminal chart):
```
6│sepal_width                                                      ┌───────────┐
 │                                                                 │sepal_width│
 │                                                                 └───────────┘
 │
 │
 │
 │
 │
 │                                ⠠
 │                             ⡀
 │                       ⠂          ⢀
4│                     ⡀     ⠂    ⢀                                      ⢀   ⢀
 │                     ⠄   ⠄ ⠄
 │           ⠈     ⠁ ⠅ ⠄ ⠄     ⠄                                ⠁
 │           ⠈   ⠁   ⠅ ⠅ ⠁   ⠁          ⠈   ⠈ ⠨       ⠄
 │       ⠈   ⢈ ⠈ ⡀ ⡀ ⠁                ⠈         ⢈ ⠁   ⡀ ⠁ ⡁ ⠁   ⠁
 │     ⠐ ⢐       ⠂ ⠂ ⠂       ⠂  ⢐ ⢐   ⠐ ⢐ ⢐ ⢀ ⢀ ⢀ ⠂ ⡂ ⠂ ⠂     ⠂ ⠂⢀     ⠐ ⠐
 │                       ⡀      ⢐ ⠐ ⢐   ⢀ ⠐ ⠐ ⢐ ⢐ ⠂     ⠂          ⠐     ⠐
 │                             ⠂  ⠐ ⠐     ⠐                              ⠐
 │                 ⠅   ⠁       ⠅⠈ ⠈           ⠈       ⠁
 │         ⠈         ⠁         ⠁        ⠠   ⠠ ⠈
2│                   ⡀                                              sepal_length
 └──────────────────────────────────────────────────────────────────────────────
 4                                       6                                     8
```

### 2. Grouped by Category (Color)

Use the `--color` option to group points by a categorical column. Each unique value gets a different color.

```bash
tva plot point tests/data/plot/iris.tsv -x petal_length -y petal_width --color label
```
![iris scatter plot with color](images/plot_point_iris_color.png)

This creates a scatter plot with three colors, one for each iris species (setosa, versicolor, virginica).

The output will show three distinct clusters with different markers/colors:
*   **Setosa**: Small petals, clustered at bottom-left
*   **Versicolor**: Medium petals, in the middle
*   **Virginica**: Large petals, at top-right

### 3. Line Chart

Use the `-l` or `--line` flag to connect points with lines instead of drawing individual points.

```bash
tva plot point tests/data/plot/iris.tsv -x sepal_length -y sepal_width --line
```

![iris line plot](images/plot_point_iris_line.png)

### 4. Using Column Indices

You can use 1-based column indices instead of header names:

```bash
tva plot point tests/data/plot/iris.tsv -x 1 -y 3 --color 5
```

This maps:
*   Column 1 (`sepal_length`) to X-axis
*   Column 3 (`petal_length`) to Y-axis
*   Column 5 (`label`) to color

### 5. Custom Chart Size

Control the chart dimensions with `--cols` and `--rows`:

```bash
tva plot point tests/data/plot/iris.tsv -x sepal_length -y sepal_width --cols 100 --rows 30
```


### 6. Different Marker Styles

Choose from three marker types with `-m` or `--marker`:

```bash
# Braille markers (default, highest resolution)
tva plot point tests/data/plot/iris.tsv -x sepal_length -y sepal_width -m braille

# Dot markers
tva plot point tests/data/plot/iris.tsv -x sepal_length -y sepal_width -m dot

# Block markers
tva plot point tests/data/plot/iris.tsv -x sepal_length -y sepal_width -m block
```

### 7. Regression Line

Use `--regression` to overlay a linear regression line (least squares fit) on the scatter plot. This helps visualize trends in the data.

```bash
tva plot point tests/data/plot/iris.tsv -x sepal_length -y petal_length -m dot --regression
```

When combined with `--color`, a separate regression line is drawn for each group:

```bash
tva plot point tests/data/plot/iris.tsv -x sepal_length -y petal_length -m dot  --color label --regression
```

Note: `--regression` cannot be used with `--line` or `--path`.

### 8. Handling Invalid Data

Use `--ignore` to skip rows with non-numeric values:

```bash
tva plot point data.tsv -x value1 -y value2 --ignore
```

## Detailed Options

| Option | Description |
| :--- | :--- |
| `-x <COL>` / `--x <COL>` | **Required.** Column for X-axis position. |
| `-y <COL>` / `--y <COL>` | **Required.** Column for Y-axis position. |
| `--color <COL>` | Column for grouping/coloring by category. |
| `-l` / `--line` | Draw line chart instead of scatter plot. |
| `--path` | Draw path chart (connect points in original order). |
| `-r` / `--regression` | Overlay linear regression line. |
| `-m <TYPE>` / `--marker <TYPE>` | Marker style: `braille` (default), `dot`, or `block`. |
| `--cols <N>` | Chart width in characters or ratio (default: `1.0`, i.e., full terminal width). |
| `--rows <N>` | Chart height in characters or ratio (default: `1.0`, i.e., full terminal height minus 1 for prompt). |
| `--ignore` | Skip rows with non-numeric values. |

## Comparison with R `ggplot2`

| Feature | `ggplot2::geom_point` | `tva plot point` |
| :--- | :--- | :--- |
| Basic scatter plot | `aes(x, y)` | `-x <col> -y <col>` |
| Color by group | `aes(color = group)` | `--color <col>` |
| Line chart | `geom_line()` | `--line` |
| Path chart | `geom_path()` | `--path` |
| Regression line | `geom_smooth(method = "lm")` | `--regression` |
| Faceting | `facet_wrap()` / `facet_grid()` | Not supported |
| Themes | `theme_*()` | Terminal-based only |
| Output | Graphics file / Viewer | Terminal ASCII/Unicode |

`tva plot point` brings the core concepts of the grammar of graphics to the command line, allowing for quick data exploration without leaving your terminal.

## Tips

1. **Large datasets**: For very large datasets, consider sampling first:
   ```bash
   tva sample data.tsv -n 1000 | tva plot point -x x -y y
   ```

2. **Piping data**: You can pipe data from other `tva` commands:
   ```bash
   tva filter data.tsv -H -c value -gt 0 | tva plot point -x x -y y
   ```

3. **Viewing output**: The chart is rendered directly to stdout. Use a terminal with good Unicode support for best results with Braille markers.
