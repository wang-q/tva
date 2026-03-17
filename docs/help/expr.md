# expr

Evaluates the expr language for each row.

Behavior:

* Parses and evaluates an expression against each row of input data.
* Default mode outputs only the expression result (original row data is not included).
* Supports arithmetic, string, logical operations, function calls, and lambda expressions.
* See `tva --help-expr` for a quick reference to the expr language and the detailed CLI instructions.

Input:

* Reads from files or standard input.
* Files ending in `.gz` are transparently decompressed.

Output:

* Default: outputs the evaluated result for each row.
* Use `-m` flag to change output mode: `eval` (default), `add`, `mutate`, `skip-null`, `filter`.

Header behavior:

* Supports basic header mode. See `tva --help-headers` for details.
* When headers are enabled, column names can be referenced with `@name` syntax.
* The output header is determined by the expression:
    * `as @name` binding: uses `name` as the header
    * `@column_name` reference: uses `column_name` as the header
    * `@1` with input headers: uses the first input column name
    * Other expressions: uses the formatted last expression string

Examples:

1. Simple arithmetic:
   `tva expr -E '2 + 3 * 4'`

2. Calculate total from price and quantity:
   `tva expr -H -E '@price * @qty' data.tsv`

3. Named output column with `as`:
   `tva expr -H -E '@price * @qty as @total' data.tsv`

4. Chain functions with pipe:
   `tva expr -H -E '@name | trim() | upper()' data.tsv`

5. Conditional expression:
   `tva expr -H -E 'if(@score >= 70, "pass", "fail")' data.tsv`

6. Add new column(s) to original row:
   `tva expr -H -m add -E '@price * @qty as @total' data.tsv`

7. Mutate (modify) existing column value:
   `tva expr -H -m mutate -E '@age + 1 as @age' data.tsv`

8. Filter rows by condition:
   `tva expr -H -m filter -E '@age > 25' data.tsv`

9. Skip null results:
   `tva expr -H -m skip-null -E 'if(@score >= 70, @name, null)' data.tsv`

10. Test with inline row data:
    `tva expr -n 'price,qty' -r '100,2' -E '@price * @qty'`
