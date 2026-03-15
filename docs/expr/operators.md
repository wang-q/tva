# Expression Operators

## Operator Precedence (high to low)

1. `()` - Grouping
2. `-` (unary) - Negation
3. `**` - Power
4. `*`, `/`, `%` - Multiply, Divide, Modulo
5. `+`, `-` (binary) - Add, Subtract
6. `++` - String concatenation
7. `==`, `!=`, `<`, `<=`, `>`, `>=` - Numeric comparison
8. `eq`, `ne`, `lt`, `le`, `gt`, `ge` - String comparison
9. `not` - Logical NOT
10. `and` - Logical AND
11. `or` - Logical OR
12. `|` - Pipe

## Arithmetic Operators

- `-x`: Negation
- `a + b`: Addition
- `a - b`: Subtraction
- `a * b`: Multiplication
- `a / b`: Division
- `a % b`: Modulo
- `a ** b`: Power

## String Operators

- `a ++ b`: String concatenation

## Comparison Operators

**Numeric:**

- `a == b`: Equal
- `a != b`: Not equal
- `a < b`: Less than
- `a <= b`: Less than or equal
- `a > b`: Greater than
- `a >= b`: Greater than or equal

**String:**

- `a eq b`: String equal
- `a ne b`: String not equal
- `a lt b`: String less than (lexicographic)
- `a le b`: String less than or equal
- `a gt b`: String greater than
- `a ge b`: String greater than or equal

**Note**: No implicit type conversion. Use string comparison operators for string comparison.

**Note**: Empty fields are treated as `null`. See [Null Type and Empty Fields](literals.md#null-type-and-empty-fields) for details.

## Logical Operators

- `not a`: Logical NOT
- `a and b`: Logical AND (short-circuit evaluation)
- `a or b`: Logical OR (short-circuit evaluation)

**Short-circuit evaluation**: `and` and `or` use short-circuit evaluation, meaning the right operand is only evaluated if necessary.

```bash
# AND short-circuit: right side not evaluated when left is false
tva expr -E 'false and print("hello")'   # Output: false (print not called)
tva expr -E 'true and print("hello")'    # Output: hello\ntrue

# OR short-circuit: right side not evaluated when left is true
tva expr -E 'true or print("hello")'     # Output: true (print not called)
tva expr -E 'false or print("hello")'    # Output: hello\ntrue

# Avoid division by zero
# If @2 is 0, the division is skipped
tva expr -E '@2 != 0 and @1 / @2 > 2' -r '100,0' -r '100,5'   # Output: false, true

# Check before accessing
# Only calculate length if @name is not empty
tva expr -E '@name != "" and len(@name) > 5' -n 'name' -r '' -r 'Alice' -r 'Alexander'  # Output: false, false, true

# Logical OR with string (returns boolean)
# Empty field is null (falsy), non-empty is truthy
tva expr -E '@email or "fallback"' -n 'email' -r '' -r 'user@example.com'  # Output: true, true

# For default value, use if() with null check:
tva expr -E 'if(@email == null, "no-email@example.com", @email)' -n 'email' -r '' -r 'user@example.com'  # Output: no-email@example.com, user@example.com
```

## Pipe Operator

- `a | f()`: Pipe left value to function (left value becomes first argument)
- `a | f(_, arg)`: Pipe with placeholder
