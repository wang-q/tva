# eval

Evaluates an expression for testing and debugging purposes.

Behavior:
*   Parses and evaluates a single expression against provided test data.
*   Does not process files; use this command to test expressions before using them in other commands.

Expression syntax:
*   Column references: `@1`, `@2` (1-based) or `@name` (when headers provided)
*   Arithmetic: `+`, `-`, `*`, `/`, `%`, `**`
*   Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
*   Logical: `and`, `or`, `not`
*   Functions: `trim()`, `upper()`, `lower()`, `len()`, `abs()`, `round()`, `min()`, `max()`, `if()`, `default()`

Examples:

1.  Test arithmetic expression:
    `tva eval '10 + 20'`

2.  Test with column references:
    `tva eval -H 'price,qty' -r '100,2' '@price * @qty'`

3.  Test multiple rows:
    `tva eval -H 'price,qty' -r '100,2' -r '200,3' '@price * @qty'`

4.  Test string functions:
    `tva eval -H 'name' -r '  alice  ' -r '  bob  ' 'upper(trim(@name))'`

5.  Test conditional expression:
    `tva eval -H 'score' -r '85' 'if(@score >= 70, "pass", "fail")'`
