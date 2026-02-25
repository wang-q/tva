# stats

Calculates summary statistics (like tsv-summarize).

Notes:
*   Supports plain text and gzipped (`.gz`) TSV files.
*   Reads from stdin if no input file is given.
*   Supports grouping by one or multiple fields.

Examples:
1. Calculate basic stats for a column:
   `tva stats docs/data/us_rent_income.tsv --header --mean estimate --max estimate`

2. Group by a column:
   `tva stats docs/data/us_rent_income.tsv -H --group-by variable --mean estimate`

3. Count rows per group:
   `tva stats docs/data/us_rent_income.tsv -H --group-by NAME --count`
