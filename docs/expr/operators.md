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

## Logical Operators

- `not a`: Logical NOT
- `a and b`: Logical AND
- `a or b`: Logical OR

## Pipe Operator

- `a | f()`: Pipe left value to function (left value becomes first argument)
- `a | f(_, arg)`: Pipe with placeholder
