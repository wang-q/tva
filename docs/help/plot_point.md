# plot point

Draws a scatter plot or line chart in the terminal.

Behavior:
*   Maps TSV columns to visual aesthetics (position, color).
*   Supports scatter plots (default) or line charts (`--line`).
*   Automatically calculates axis ranges from data (overridable with `--xlim`, `--ylim`).

Input:
*   Reads from files or standard input.
*   Files ending in `.gz` are transparently decompressed.
*   Assumes the first line is a header row with column names.

Output:
*   Renders an ASCII/Unicode chart to standard output.
*   Chart dimensions can be controlled with `--cols` and `--rows`.

Aesthetics:
*   `-x`, `--x`: Column for X-axis position (required).
*   `-y`, `--y`: Column for Y-axis position (required).
*   `--color`: Column for grouping/coloring points by category.

Options:
*   `-l`, `--line`: Connect points with lines (default is scatter plot).
*   `-m`, `--marker`: Marker style - `braille` (default), `dot`, or `block`.
*   `--cols`: Chart width in characters (default: 80).
*   `--rows`: Chart height in characters (default: 24).
*   `--xlim`: X-axis range as `min,max` (e.g., `0,100`).
*   `--ylim`: Y-axis range as `min,max` (e.g., `0,100`).
*   `--ignore`: Skip rows with non-numeric values in X/Y columns.

Column specification:
*   Columns can be specified by 1-based index or header name.
*   Run `tva --help-fields` for field syntax details.

Examples:
1.  Basic scatter plot:
    `tva plot point data.tsv -x age -y income`

2.  Grouped by category:
    `tva plot point iris.tsv -x petal_length -y petal_width --color label`

3.  Line chart with custom size:
    `tva plot point timeseries.tsv -x time -y value --line --cols 100 --rows 30`

4.  Fixed axis ranges:
    `tva plot point data.tsv -x x -y y --xlim 0,10 --ylim -5,5`

5.  Using column indices:
    `tva plot point data.tsv -x 1 -y 3 --color 5`
