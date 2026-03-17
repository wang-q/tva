# expr

Evaluates the expr language for each row and outputs only the result.

Behavior:

* Parses and evaluates an expression against each row of input data.
* Outputs only the expression result for each row (original row data is not included).
* Supports arithmetic, string, logical operations, function calls, and lambda expressions.
* See `tva --help-expr` for a quick reference to the expr language.

Input:

* Reads from files or standard input.
* Files ending in `.gz` are transparently decompressed.

Output:

* Outputs the evaluated result for each row (original row data is not included).

Header behavior:

* Supports basic header mode. See `tva --help-headers` for details.
* When headers are enabled, column names can be referenced with `@name` syntax.
* The output header is determined by the expression:
    * `as @name` binding: uses `name` as the header
    * `@column_name` reference: uses `column_name` as the header
    * `@1` with input headers: uses the first input column name
    * Other expressions: uses the formatted last expression string

Examples:

1. Simple arithmetic (no input needed):
   `tva expr -E '2 + 3 * 4'`

2. Calculate factorial using reduce (no input needed):
   `tva expr -E 'reduce(range(1, 11), 1, (acc, n) => acc * n)'`

3. Calculate total from price and quantity:
   `tva expr -H -E '@price * @qty' data.tsv`

4. Named output column with `as`:
   `tva expr -H -E '@price * @qty as @total' data.tsv`

5. Test with inline row data:
   `tva expr -n 'price,qty' -r '100,2' -r '200,3' -E '@price * @qty'`

6. Chain functions with pipe:
   `tva expr -H -E '@description | trim() | lower()' data.tsv`

7. Use underscore in multi-argument functions:
   `tva expr -H -E '@name | substr(_, 0, 3)' data.tsv`

8. Conditional expression:
   `tva expr -H -E 'if(@score >= 70, "pass", "fail")' data.tsv`

9. Skip null results (filtering):
   `tva expr -H -m skip-null -E 'if(@score >= 70, @name, null)' data.tsv`

10. Filter rows by condition (preserves original row and header):
    `tva expr -H -m filter -E '@age > 25' data.tsv`

11. Running total with persistent variables:
    `tva expr -H -E '@amount + @__total as @__total; @__total' data.tsv`
