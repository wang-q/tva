# plot box

Draws a box plot (box-and-whisker plot) showing the distribution of continuous variable(s).

Behavior:

* Visualizes five summary statistics for each group:
    * **Min**: Lower whisker (smallest non-outlier value)
    * **Q1**: First quartile (25th percentile) - bottom of the box
    * **Median**: Second quartile (50th percentile) - line inside the box
    * **Q3**: Third quartile (75th percentile) - top of the box
    * **Max**: Upper whisker (largest non-outlier value)
* Outliers are values beyond 1.5 * IQR (inter-quartile range) from the quartiles.

Input:

* Reads from files or standard input.
* Files ending in `.gz` are transparently decompressed.
* Assumes the first line is a header row with column names.

Output:

* Renders a box plot to the terminal.

Examples:

1. Draw a simple box plot
   `tva plot box -y age data.tsv`

2. Draw box plots by category
   `tva plot box -y age --color species data.tsv`

3. Show outliers beyond the whiskers
   `tva plot box -y age --outliers data.tsv`

4. Plot multiple columns
   `tva plot box -y value1,value2 data.tsv`
