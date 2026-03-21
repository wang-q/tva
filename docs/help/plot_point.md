# plot point

Draws a scatter plot, line chart, or path chart in the terminal.

Behavior:

* Maps TSV columns to visual aesthetics (position, color).
* Supports scatter plots (default), line charts (`--line`), or path charts (`--path`).
* Supports overlaying linear regression lines (`--regression`).
* `--regression` cannot be used with `--line` or `--path`.

Input:

* Reads from files or standard input.
* Files ending in `.gz` are transparently decompressed.
* Assumes the first line is a header row with column names.

Output:

* Renders an ASCII/Unicode chart to standard output.
* Chart dimensions can be controlled with `--cols` and `--rows`.

Chart types:

* Scatter plot (default): Individual points without connecting lines.
* `--line` / `-l`: Connect points with lines, sorted by X value (good for trends).
* `--path`: Connect points with lines, preserving original data order (good for trajectories).
* `--regression` / `-r`: Overlay linear regression line (least squares fit).
  Cannot be used with `--line` or `--path`.

Examples:

1. Basic scatter plot
   `tva plot point data.tsv -x age -y income`

2. Grouped by category
   `tva plot point iris.tsv -x petal_length -y petal_width --color label`

3. Line chart (sorted by X, good for trends)
   `tva plot point timeseries.tsv -x time -y value --line --cols 100 --rows 30`

4. Path chart (preserves data order, good for trajectories)
   `tva plot point trajectory.tsv -x x -y y --path --cols 100 --rows 30`

5. With regression line (linear fit)
   `tva plot point iris.tsv -x sepal_length -y petal_length --regression`

6. Using column indices
   `tva plot point data.tsv -x 1 -y 3 --color 5`

7. Multiple Y columns
   `tva plot point data.tsv -x time -y value1,value2`
