# Expression Literals and Variables

## Literal Syntax

| Type | Syntax | Examples |
| :--- | :--- | :--- |
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

# String literals
tva expr -E '"hello" ++ " " ++ "world"'  # Returns: hello world

# Boolean literals
tva expr -E 'true and false'      # Returns: false

# Null literal
tva expr -E 'default(null, "fallback")'  # Returns: fallback

# List literal
tva expr -E '[1, 2, 3]'           # Returns: [1, 2, 3]

# Lambda literal
tva expr -E 'map([1, 2, 3], x => x * 2)'  # Returns: [2, 4, 6]
```

## Type System

TVA uses a dynamic type system with automatic type recognition at runtime:

| Type | Description | Conversion Rules |
| :--- | :--- | :--- |
| `Int` | 64-bit signed integer | Returns `null` on string parse failure |
| `Float` | 64-bit floating point | Integers automatically promoted to float |
| `String` | UTF-8 string | Numbers/booleans can be explicitly converted |
| `Bool` | Boolean value | Empty string, 0, `null` are falsy |
| `Null` | Null value | Represents missing or invalid data |
| `List` | Heterogeneous list | Elements can be any type |

## Type Conversion

- **Explicit conversion**: Use `int()`, `float()`, `string()` functions
- **Numeric operations**: Mixed int/float operations promote result to float
- **String concatenation**: `++` operator converts operands to strings
- **Comparison**: Same-type comparison only; different types always return `false`

## String Escape Sequences

| Escape | Meaning | Example |
| :--- | :--- | :--- |
| `\n` | Newline | `"line1\nline2"` |
| `\t` | Tab | `"col1\tcol2"` |
| `\r` | Carriage return | `"\r\n"` (Windows line ending) |
| `\\` | Backslash | `"C:\\Users\\name"` |
| `\"` | Double quote | `"say \"hello\""` |
| `\'` | Single quote | `'it\'s ok'` |

## Column References

Use `@` prefix to reference columns, avoiding conflicts with Shell variables:

| Syntax | Description | Example |
| :--- | :--- | :--- |
| `@0` | Entire row content | `@0` represents all columns |
| `@1`, `@2` | 1-based column index | `@1` represents the first column |
| `@col_name` | Column name reference | `@price` represents the price column |
| ` @"col name"` | Column name with spaces | ` @"user name"` represents column "user name" |

**Design rationale**:
- **Shell-friendly**: `@` has no special meaning in bash/zsh, no escaping needed
- **Concise**: Only 2 characters (`Shift+2`)

**Type behavior**:
- Column references return `String` by default (raw bytes)
- Numeric operations automatically attempt parsing; failure yields `null`
- Use `int(@col)` or `float(@col)` for explicit type specification

## Variable Binding

Use `as` keyword to bind expression results to variables for reuse in subsequent pipes:

```bash
# Basic syntax: bind calculation result
tva expr -n "price,qty,tax_rate" -r "10,5,0.1" -E '@price * @qty as @total; @total * (1 + @tax_rate)'
# Returns: 55

# Reuse intermediate results
tva expr -n "name" -r "John Smith" -E '@name | split(" ") as @parts; first(@parts) ++ "." ++ last(@parts)'
# Returns: John.Smith

# Multiple variable bindings
tva expr -n "price,qty" -r "10,5" -E '@price as @p; @qty as @q; @p * @q'
# Returns: 50
```

**Rules**:
- Variables are valid within the current row only, cleared when entering next row
- Can shadow column references (`@price as @price`)
- Column references check variables first, then fall back to column name lookup

**Design notes**:
- `as` aligns with pipe semantics ("name the result from the left...")
- Unified `@` prefix reduces cognitive burden
- References jq syntax but removes `$` to avoid Shell conflicts
