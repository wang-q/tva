# expr

Evaluates an expression for each row and outputs only the result.

Behavior:

* Parses and evaluates an expression against each row of input data.
* Outputs only the expression result for each row (original row data is not included).
* Supports arithmetic, string, logical operations, function calls, and lambda expressions.
* Variables persist across rows within the same file.

Input:

* Reads from files or standard input.
* Files ending in `.gz` are transparently decompressed.

Output:

* Outputs the expression result for each row (original row data is not included).

Header behavior:

* Supports all four header modes. See `tva --help-headers` for details.
* When headers are enabled, column names can be referenced with `@name` syntax.
* The output header is determined by the expression:
    * `as @name` binding: uses `name` as the header
    * `@column_name` reference: uses `column_name` as the header
    * `@1` with input headers: uses the first input column name
    * Other expressions: uses the formatted last expression string

Expression syntax:

* Column references: `@1`, `@2` (1-based) or `@name` (when headers provided)
* Whole row reference: `@0` (all columns as a list)
* Variables: `@var_name` (bound by `as`, persists across rows)
* Global variables: `@__index`, `@__file`, `@__row` (built-in)
* Arithmetic: `+`, `-`, `*`, `/`, `%`, `**`
* Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
* String comparison: `eq`, `ne`, `lt`, `le`, `gt`, `ge`
* Logical: `and`, `or`, `not`
* String concatenation: `++`
* Functions: `trim()`, `upper()`, `lower()`, `len()`, `abs()`, `round()`, `min()`, `max()`, `if()`,
  `default()`, `substr()`, `replace()`, `split()`, `join()`, `range()`, `map()`, `filter()`, `reduce()`
* Pipe operator: `|` for chaining functions (e.g., `@name | trim() | upper()`)
* Underscore placeholder: `_` for piped values in multi-argument functions (e.g., `@name | substr(_, 0, 3)`)
* Lambda expressions: `x => x + 1` or `(x, y) => x + y`
* List literals: `[1, 2, 3]` or `[@a, @b, @c]`
* Variable binding: `as` for intermediate results (e.g., `@price * @qty as @total; @total * 0.9`)
* Method call syntax: `@name.upper()`, `@num.abs()`

Examples:

1. Simple arithmetic (no input needed):
   `tva expr -E '2 + 3 * 4'`

2. Calculate factorial of 10 using reduce (no input needed):
   `tva expr -E 'reduce(range(1, 11), 1, (acc, n) => acc * n)'`

3. Calculate total from price and quantity (with input file):
   `tva expr -H -E '@price * @qty' data.tsv`

4. Calculate total with named output column:
   `tva expr -H -E '@price * @qty as @total' data.tsv`

5. Test with inline row data:
   `tva expr -n 'price,qty' -r '100,2' -r '200,3' -E '@price * @qty'`

6. Apply string transformations:
   `tva expr -H -E 'upper(trim(@name))' data.tsv`

7. Use pipe operator for chaining:
   `tva expr -H -E '@description | trim() | lower()' data.tsv`

8. Use underscore placeholder in pipes:
   `tva expr -H -E '@name | substr(_, 0, 3)' data.tsv`

9. Conditional expression:
   `tva expr -H -E 'if(@score >= 70, "pass", "fail")' data.tsv`

10. Variable binding for complex calculations:
    `tva expr -H -E '@price * @qty as @total; @total * 0.9 as @discounted; @discounted' data.tsv`

11. Skip rows where result is null (filtering):
    `tva expr -H --skip-null -E 'if(@score >= 70, @name, null)' data.tsv`

12. Use map with lambda:
    `tva expr -H -E '@numbers | split(",") | map(_, n => n + 1) | join(",")' data.tsv`

13. Filter a list:
    `tva expr -H -E '@items | split(",") | filter(_, x => x != "bad") | join(",")' data.tsv`

14. Running total using persistent variables:
    `tva expr -H -E '@amount + @__total as @__total; @__total' data.tsv`
