# bin2d

Creates a 2D binning heatmap (density plot) of two numeric columns.

Divides the plane into rectangular bins, counts the number of points in
each bin, and visualizes the density using character intensity.

Input:
*   Reads from files or standard input.
*   Files ending in `.gz` are transparently decompressed.

Output:
*   Renders a heatmap to the terminal using character density.
*   Density scale (low to high):
    *   · ≥5% (dark grey)
    *   ░ ≥20% (grey)
    *   ▒ ≥40% (white)
    *   ▓ ≥60% (yellow)
    *   █ ≥80% (red)
*   Values below 5% are not shown.

Header behavior:
*   Supports all four header modes. See `tva --help-headers` for details.

Examples:

1.  Basic 2D binning:

        tva plot bin2d data.tsv -x age -y income

2.  Specify number of bins:

        tva plot bin2d data.tsv -x age -y income --bins 20

3.  Different bins for x and y:

        tva plot bin2d data.tsv -x age -y income --bins 30,10

4.  Use automatic bin count strategy:

        tva plot bin2d data.tsv -x age -y income -S freedman-diaconis

5.  Specify bin width:

        tva plot bin2d data.tsv -x age -y income --binwidth 5

6.  Custom chart size:

        tva plot bin2d data.tsv -x age -y income --cols 100 --rows 30
