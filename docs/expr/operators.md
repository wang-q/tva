# Expression Operators

TVA provides a comprehensive set of operators for arithmetic, string, comparison, and logical operations.

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

### Concatenation

`a ++ b` - Concatenates two values as strings.

```bash
tva expr -E '"hello" ++ " " ++ "world"'  # Returns: "hello world"
tva expr -E '"count: " ++ 42'            # Returns: "count: 42"
tva expr -E '1 ++ 2 ++ 3'                 # Returns: "123"
```

Both operands are converted to strings before concatenation.

## Comparison Operators

### Numeric Comparison

Compare numbers. Returns boolean.

| Operator | Description | Example |
|:---------|:------------|:--------|
| `==` | Equal | `5 == 5` → `true` |
| `!=` | Not equal | `5 != 3` → `true` |
| `<` | Less than | `3 < 5` → `true` |
| `<=` | Less than or equal | `5 <= 5` → `true` |
| `>` | Greater than | `5 > 3` → `true` |
| `>=` | Greater than or equal | `5 >= 3` → `true` |

```bash
tva expr -E '5 == 5'                # Returns: true
tva expr -E '10 > 5'                # Returns: true
tva expr -E '@1 > 100' -r '150'     # Returns: true
```

**Note**: Different types always compare as not equal.

```bash
tva expr -E '5 == "5"'              # Returns: false (int vs string)
tva expr -E '5 == 5.0'              # Returns: true (numeric comparison)
```

### String Comparison

Lexicographic string comparison. Returns boolean.

| Operator | Description | Example |
|:---------|:------------|:--------|
| `eq` | String equal | `"a" eq "a"` → `true` |
| `ne` | String not equal | `"a" ne "b"` → `true` |
| `lt` | String less than | `"a" lt "b"` → `true` |
| `le` | String less than or equal | `"a" le "a"` → `true` |
| `gt` | String greater than | `"b" gt "a"` → `true` |
| `ge` | String greater than or equal | `"b" ge "a"` → `true` |

```bash
tva expr -E '"apple" lt "banana"'   # Returns: true
tva expr -E '"hello" eq "hello"'    # Returns: true
```

**Note**: Use string comparison operators for string comparison, not `==`.

```bash
# Correct: string comparison
tva expr -E '"10" lt "2"'           # Returns: true (lexicographic)

# Incorrect: numeric comparison with strings
tva expr -E '"10" == "10"'          # Returns: true
tva expr -E '"10" < "2"'            # Returns: false (parsed as numbers)
```

### Null Handling

Empty fields are treated as `null`. See [Null Type and Empty Fields](literals.md#null-type-and-empty-fields) for details.

```bash
tva expr -E '@1 == null' -r ''      # Returns: true (empty field)
tva expr -E '@1 == ""' -r ''        # Returns: false (null != "")
```

## Logical Operators

### Logical NOT

`not a` - Negates a boolean value.

```bash
tva expr -E 'not true'              # Returns: false
tva expr -E 'not false'             # Returns: true
tva expr -E 'not @1' -r ''          # Returns: true (null is falsy)
```

### Logical AND

`a and b` - Returns true if both operands are true.

```bash
tva expr -E 'true and true'         # Returns: true
tva expr -E 'true and false'        # Returns: false
tva expr -E '5 > 3 and 10 < 20'     # Returns: true
```

**Short-circuit evaluation**: The right operand is only evaluated if the left is true.

```bash
# Right side not evaluated when left is false
tva expr -E 'false and print("hello")'   # Returns: false (print not called)
tva expr -E 'true and print("hello")'    # Prints: hello, returns: true
```

### Logical OR

`a or b` - Returns true if either operand is true.

```bash
tva expr -E 'true or false'         # Returns: true
tva expr -E 'false or false'        # Returns: false
tva expr -E '5 > 10 or 3 < 5'       # Returns: true
```

**Short-circuit evaluation**: The right operand is only evaluated if the left is false.

```bash
# Right side not evaluated when left is true
tva expr -E 'true or print("hello")'     # Returns: true (print not called)
tva expr -E 'false or print("hello")'    # Prints: hello, returns: true
```

### Practical Examples

```bash
# Avoid division by zero
# If @2 is 0, the division is skipped due to short-circuit
tva expr -E '@2 != 0 and @1 / @2 > 2' -r '100,0' -r '100,5'
# Returns: false, true

# Check before accessing
# Only calculate length if @name is not empty
tva expr -E '@name != null and len(@name) > 5' -n 'name' -r '' -r 'Alice' -r 'Alexander'
# Returns: false, false, true

# Default value with or
# Note: returns boolean, not the value
tva expr -E '@email or true' -n 'email' -r '' -r 'user@example.com'
# Returns: true, true

# For actual default value, use if() or default():
tva expr -E 'if(@email == null, "no-email@example.com", @email)' -n 'email' -r '' -r 'user@example.com'
# Returns: no-email@example.com, user@example.com
```

## Pipe Operator

`a | f()` - Passes the left value as the first argument to the function on the right.

### Single Argument Functions

For functions that take one argument, the pipe value is used directly:

```bash
tva expr -E '"hello" | upper()'           # Returns: HELLO
tva expr -E '[1, 2, 3] | reverse()'       # Returns: [3, 2, 1]
tva expr -E '@name | trim() | lower()'    # Chain multiple pipes
```

### Multiple Argument Functions

Use `_` as a placeholder for the piped value:

```bash
tva expr -E '"hello world" | substr(_, 0, 5)'    # Returns: hello
tva expr -E '"a,b,c" | split(_, ",")'            # Returns: ["a", "b", "c"]
tva expr -E '"hello" | replace(_, "l", "x")'     # Returns: hexxo
```

### Complex Pipelines

Combine multiple operations:

```bash
# Data transformation
tva expr -n "data" -r "1,2,3,4,5" -E '
    @data
    | split(_, ",")
    | map(_, x => int(x) * 2)
    | join(_, "-")
'
# Returns: "2-4-6-8-10"

# Validation pipeline
tva expr -n "email" -r "  Test@Example.COM  " -E '
    @email
    | trim()
    | lower()
    | regex_match(_, ".*@.*\\.com")
'
# Returns: true
```

## Operator Precedence Examples

```bash
# Without parentheses: multiplication before addition
tva expr -E '2 + 3 * 4'             # Returns: 14 (not 20)

# With parentheses: force addition first
tva expr -E '(2 + 3) * 4'           # Returns: 20

# Comparison before logical
tva expr -E '5 > 3 and 10 < 20'     # Returns: true

# Pipe has lowest precedence
tva expr -E '1 + 2 | int()'         # Returns: 3 (not error)
```

## Best Practices

1. **Use parentheses for clarity**: Even when not strictly necessary, parentheses make intent clear
2. **Prefer string operators for strings**: Use `eq` instead of `==` for string comparison
3. **Use short-circuit for safety**: `not @col or expensive_operation()`
4. **Chain with pipes**: `@data | trim() | lower()` is more readable than `lower(trim(@data))`
