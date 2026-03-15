# Expression Literals

## Literal Syntax

| Type    | Syntax                    | Examples                        |
|:--------|:--------------------------|:--------------------------------|
| Integer | Digit sequence            | `42`, `-10`, `1_000_000`        |
| Float   | Decimal point or exponent | `3.14`, `-0.5`, `1e10`          |
| String  | Single or double quotes   | `"hello"`, `'world'`            |
| Boolean | `true` / `false`          | `true`, `false`                 |
| Null    | `null`                    | `null`                          |
| List    | Square brackets           | `[1, 2, 3]`, `["a", "b"]`       |
| Lambda  | Arrow function            | `x => x + 1`, `(x, y) => x + y` |

```bash
# Integer and float literals
tva expr -E '42 + 3.14'           # Returns: 45.14

# String literals
tva expr -E '"hello" ++ " " ++ "world"'  # Returns: hello world

# Boolean literals
tva expr -E 'true and false'      # Returns: false

# Null literal
tva expr -E 'default(null, "fallback")'  # Returns: fallback

# List literal
tva expr -E '[1, 2, 3]'           # Returns: [1, 2, 3]
tva expr -E '[[1,2], "string", true, null, -5]'

# Lambda literal
tva expr -E 'map([1, 2, 3], x => x * 2)'  # Returns: [2, 4, 6]
```

## Type System

TVA uses a dynamic type system with automatic type recognition at runtime. Since TSV files store all
data as strings, TVA automatically converts values to appropriate types during expression
evaluation:

| Type     | Description           | Conversion Rules                             |
|:---------|:----------------------|:---------------------------------------------|
| `Int`    | 64-bit signed integer | Returns `null` on string parse failure       |
| `Float`  | 64-bit floating point | Integers automatically promoted to float     |
| `String` | UTF-8 string          | Numbers/booleans can be explicitly converted |
| `Bool`   | Boolean value         | Empty string, 0, `null` are falsy            |
| `Null`   | Null value            | Represents missing or invalid data           |
| `List`   | Heterogeneous list    | Elements can be any type                     |

- **Explicit conversion**: Use `int()`, `float()`, `string()` functions
- **Numeric operations**: Mixed int/float operations promote result to float
- **String concatenation**: `++` operator converts operands to strings
- **Comparison**: Same-type comparison only; different types always return `false`

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

## String Escape Sequences

| Escape | Meaning         | Example                        |
|:-------|:----------------|:-------------------------------|
| `\n`   | Newline         | `"line1\nline2"`               |
| `\t`   | Tab             | `"col1\tcol2"`                 |
| `\r`   | Carriage return | `"\r\n"` (Windows line ending) |
| `\\`   | Backslash       | `"C:\\Users\\name"`            |
| `\"`   | Double quote    | `"say \"hello\""`              |
| `\'`   | Single quote    | `'it\'s ok'`                   |
