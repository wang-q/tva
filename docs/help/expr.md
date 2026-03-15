# expr

Evaluates an expression for each row and outputs only the result.

Behavior:

* Parses and evaluates an expression against each row of input data.
* Outputs only the expression result for each row (original row data is not included).
* Supports arithmetic, string, logical operations and function calls.

Input:

* Reads from files or standard input.
* Files ending in `.gz` are transparently decompressed.

Output:

* Outputs the expression result for each row (original row data is not included).
* The header line is the formatted expression string.

Header behavior:

* Supports all four header modes. See `tva --help-headers` for details.
* When headers are enabled, column names can be referenced with `@name` syntax.

Expression syntax:

* Column references: `@1`, `@2` (1-based) or `@name` (when headers provided)
* Arithmetic: `+`, `-`, `*`, `/`, `%`, `**`
* Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
* Logical: `and`, `or`, `not`
* Functions: `trim()`, `upper()`, `lower()`, `len()`, `abs()`, `round()`, `min()`, `max()`, `if()`,
  `default()`
* Pipe operator: `|` for chaining functions (e.g., `@name | trim() | upper()`)
* Variable binding: `as` for intermediate results (e.g., `@price * @qty as @total; @total * 0.9`)

Examples:

1. Simple arithmetic (no input needed):
   `tva expr -E '2 + 3 * 4'`

2. Calculate factorial of 10 using reduce (no input needed):
   `tva expr -E 'reduce(range(1, 11), 1, (acc, n) => acc * n)'`

3. Calculate total from price and quantity (with input file):
   `tva expr -H -E '@price * @qty' data.tsv`

4. Test with inline row data:
   `tva expr -n 'price,qty' -r '100,2' -r '200,3' -E '@price * @qty'`

5. Apply string transformations:
   `tva expr -H -E 'upper(trim(@name))' data.tsv`

6. Use pipe operator for chaining:
   `tva expr -H -E '@description | trim() | lower()' data.tsv`

7. Conditional expression:
   `tva expr -H -E 'if(@score >= 70, "pass", "fail")' data.tsv`

8. Variable binding for complex calculations:
   `tva expr -H -E '@price * @qty as @total; @total * 0.9 as @discounted; @discounted' data.tsv`

9. Skip rows where result is null (filtering):
   `tva expr -H --skip-null -E 'if(@score >= 70, @name, null)' data.tsv`
