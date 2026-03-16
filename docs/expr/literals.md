# Expr Literals

Literals represent constant values in expressions. TVA supports integers, floats, strings, booleans, null, and lists.

## Literal Syntax

| Type | Syntax | Examples |
|:-----|:-------|:---------|
| Integer | Digit sequence | `42`, `-10`, `1_000_000` |
| Float | Decimal point or exponent | `3.14`, `-0.5`, `1e10` |
| String | Single or double quotes | `"hello"`, `'world'` |
| Boolean | `true` / `false` | `true`, `false` |
| Null | `null` | `null` |
| List | Square brackets | `[1, 2, 3]`, `["a", "b"]` |
| Lambda | Arrow function | `x => x + 1`, `(x, y) => x + y` |

```bash
# Integer and float literals
tva expr -E '42 + 3.14'           # Returns: 45.14
tva expr -E '1e6'                 # Returns: 1000000

# String literals
tva expr -E '"hello" ++ " " ++ "world"'  # Returns: hello world

# Boolean literals
tva expr -E 'true and false'      # Returns: false

# Null literal
tva expr -E 'default(null, "fallback")'  # Returns: fallback

# List literal
tva expr -E '[1, 2, 3]'           # Returns: [1, 2, 3]
tva expr -E '[[1,2], "string", true, null, -5]'
# Returns: [[1, 2], "string", true, null, -5]

# Lambda literal
tva expr -E 'map([1, 2, 3], x => x * 2)'  # Returns: [2, 4, 6]
```

## Type System

TVA uses a dynamic type system with automatic type recognition at runtime. Since TSV files store all
data as strings, TVA automatically converts values to appropriate types during expression
evaluation:

| Type | Description | Conversion Rules |
|:-----|:------------|:---------------|
| `Int` | 64-bit signed integer | Returns `null` on string parse failure |
| `Float` | 64-bit floating point | Integers automatically promoted to float |
| `String` | UTF-8 string | Numbers/booleans can be explicitly converted |
| `Bool` | Boolean value | Empty string, 0, `null` are falsy |
| `Null` | Null value | Represents missing or invalid data |
| `List` | Heterogeneous list | Elements can be any type |
| `DateTime` | UTC datetime | Used by datetime functions |
| `Lambda` | Anonymous function | Used with higher-order functions |

### Type Conversion

- **Explicit conversion**: Use `int()`, `float()`, `string()` functions
- **Numeric operations**: Mixed int/float operations promote result to float
- **String concatenation**: `++` operator converts operands to strings
- **Comparison**: Same-type comparison only; different types always return `false`

```bash
# Explicit type conversion
tva expr -E 'int("42")'           # Returns: 42
tva expr -E 'float("3.14")'       # Returns: 3.14
tva expr -E 'string(42)'          # Returns: "42"

# Automatic promotion in mixed operations
tva expr -E '42 + 3.14'           # Returns: 45.14 (float)
tva expr -E '10 / 4'              # Returns: 2.5 (float)
```

### Null Type and Empty Fields

In TVA, **empty fields from TSV data are treated as `null`**, not empty strings. This is important
because `null` behaves differently from `""` in expressions.

**Key behaviors:**

| Expression | Empty Field (`null`) | Non-Empty Field (`"text"`) |
|:-----------|:---------------------|:---------------------------|
| `@col == ""` | `false` | `false` |
| `@col == null` | `true` | `false` |
| `not @col` | `true` | `false` |
| `len(@col)` | `0` | length of string |

**How to check for empty values:**

```bash
# Correct way to check for empty field
tva expr -E 'not @1' -r ''              # Output: true
tva expr -E '@1 == null' -r ''          # Output: true

# Incorrect: empty field is not equal to empty string
tva expr -E '@1 == ""' -r ''            # Output: false
```

**Use case: Default values**

```bash
# Provide default value for empty field
tva expr -E 'if(@email == null, "no-email", @email)' -n 'email' -r '' -r 'user@test.com'
# Output: no-email, user@test.com
```

## String Literals

Strings can be enclosed in single or double quotes:

```bash
tva expr -E '"hello"'              # Double quotes
tva expr -E "'hello'"              # Single quotes (in shell)
```

In regular quoted strings, these escape sequences are recognized:

| Escape | Meaning | Example |
|:-------|:--------|:--------|
| `\n` | Newline | `"line1\nline2"` |
| `\t` | Tab | `"col1\tcol2"` |
| `\r` | Carriage return | `"\r\n"` (Windows line ending) |
| `\\` | Backslash | `"C:\\Users\\name"` |
| `\"` | Double quote | `q(say "hello")` (or `"say \"hello\""` in code) |
| `\'` | Single quote | `q(it's ok)` (or `'it\'s ok'` in code) |

```bash
# Using escape sequences
tva expr -E '"line1\nline2"'        # Contains newline
tva expr -E '"col1\tcol2"'          # Contains tab

```

### The `q()` string

For strings containing both single and double quotes, use the `q()` operator
(like Perl's q//). Content inside `q()` is taken literally, only `\(`, `\)`,
and `\\` need escaping:

```bash
# No need to escape quotes inside q()
tva expr -E 'q(He said "It is ok!")'     # Returns: He said "It is ok!"
tva expr -E "q(it's a 'test')"            # Returns: it's a 'test'

# For strings containing quotes, q() is often easier:
tva expr -E 'q(say "hello")'        # No need to escape quotes
tva expr -E "q(it's ok)"            # No need to escape quotes

# Escaping parentheses
tva expr -E 'q(test \(nested\) parens)'   # Returns: test (nested) parens

tva expr -H -s -E '@cut eq "Premium"' docs/data/diamonds.tsv
tva expr -H -s -E '@cut eq q(Premium)' docs/data/diamonds.tsv
```

## List Literals

Lists are ordered collections that can contain elements of any type:

```bash
# Homogeneous lists
tva expr -E '[1, 2, 3]'             # List of integers
tva expr -E '["a", "b", "c"]'       # List of strings

# Heterogeneous lists
tva expr -E '[1, "two", true, null]'  # Mixed types

# Nested lists
tva expr -E '[[1, 2], [3, 4]]'      # List of lists

# Empty list
tva expr -E '[]'                    # Empty list
```

### List Operations

Lists support various operations through functions:

```bash
# Access elements
tva expr -E 'nth([10, 20, 30], 1)'  # Returns: 20 (0-based)

# List length
tva expr -E 'len([1, 2, 3])'        # Returns: 3

# Transform
tva expr -E 'map([1, 2, 3], x => x * 2)'  # Returns: [2, 4, 6]

# Filter
tva expr -E 'filter([1, 2, 3, 4], x => x > 2)'  # Returns: [3, 4]

# Join
tva expr -E 'join(["a", "b", "c"], "-")'  # Returns: "a-b-c"
```

## Integer Literals

Integers are 64-bit signed numbers:

```bash
tva expr -E '42'                    # Positive integer
tva expr -E '-10'                   # Negative integer
tva expr -E '0'                     # Zero
```

## Float Literals

Floats are 64-bit IEEE 754 floating-point numbers:

```bash
# Decimal notation
tva expr -E '3.14'
tva expr -E '-0.5'
tva expr -E '10.0'

# Scientific notation
tva expr -E '1e10'                  # 10 billion
tva expr -E '2.5e-3'                # 0.0025
tva expr -E '-1.5E+6'               # -1,500,000
```

## Boolean Literals

Booleans represent true/false values:

```bash
tva expr -E 'true'                  # True
tva expr -E 'false'                 # False
```

Boolean values can be used in logical operations:

```bash
tva expr -E 'true and false'        # Returns: false
tva expr -E 'true or false'         # Returns: true
tva expr -E 'not true'              # Returns: false
```

## Lambda Literals

Lambdas are anonymous functions used with higher-order functions:

```bash
# Single parameter
tva expr -E 'map([1, 2, 3], x => x + 1)'

# Multiple parameters
tva expr -E 'reduce([1, 2, 3], 0, (acc, x) => acc + x)'
```
