# stats

Calculates summary statistics (like tsv-summarize).

Input:
*   If no input files are given, or an input file is 'stdin', data is read
    from standard input.
*   Files ending in '.gz' are transparently decompressed.

Grouping:
*   Supports grouping by one or multiple fields using `--group-by`.

Examples:
1.  Calculate basic stats for a column:
    `tva stats docs/data/us_rent_income.tsv --header --mean estimate --max estimate`

2.  Group by a column:
    `tva stats docs/data/us_rent_income.tsv -H --group-by variable --mean estimate`

3.  Count rows per group:
 *   `tva stats docs/data/us_rent_income.tsv -H --group-by NAME --count`

4.  List unique values in a group:
    `tva stats docs/data/us_rent_income.tsv -H --group-by variable --unique estimate`

5.  Pick a random value from a group:
    `tva stats docs/data/us_rent_income.tsv -H --group-by variable --rand estimate`
