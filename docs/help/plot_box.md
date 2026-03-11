# box

Draw a box plot (box-and-whisker plot) showing the distribution of a continuous variable.

A box plot visualizes five summary statistics:
*   **Min**: Lower whisker (smallest non-outlier value)
*   **Q1**: First quartile (25th percentile) - bottom of the box
*   **Median**: Second quartile (50th percentile) - line inside the box
*   **Q3**: Third quartile (75th percentile) - top of the box
*   **Max**: Upper whisker (largest non-outlier value)

Outliers are values beyond 1.5 * IQR (inter-quartile range) from the quartiles.

## Usage

    tva plot box [options] <infile>

## Options

*   `-y, --y <columns>`: Y axis column(s) to plot (1-based index or column name). Required.
*   `--color <column>`: Color grouping column for categorical box plots.
*   `--outliers`: Show outlier points beyond the whiskers.
*   `--cols <width>`: Chart width in characters or ratio (default: 1.0).
*   `--rows <height>`: Chart height in characters or ratio (default: 1.0).
*   `--ignore`: Ignore rows with non-numeric values.
*   `<infile>`: Input TSV file (default: stdin).

## Examples

Draw a simple box plot:

    $ tva plot box -y age data.tsv

Draw box plots by category:

    $ tva plot box -y age --color species data.tsv

Show outliers:

    $ tva plot box -y age --outliers data.tsv

## Header behavior

*   Supports all four header modes. See `tva --help-headers` for details.

## References

Based on Tukey's box plot design. See McGill, R., Tukey, J. W. and Larsen, W. A. (1978)
"Variations of box plots", The American Statistician 32, 12-16.
