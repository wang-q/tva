# TVA's expr language

The `expr` language evaluates expressions (like spreadsheet formulas) to transform TSV data.

## Quick Examples

```bash
# Basic arithmetic
tva expr -E '42 + 3.14'
# Output: 45.14

# String manipulation
tva expr -E '"hello" | upper()'
# Output: HELLO

# Using higher-order functions
tva expr -E "map([1,2,3,4,5], x => x * x)"
# Output: [1, 4, 9, 16, 25]
```

## Topics

### [Literals](literals.md)

Integer, float, string, boolean, null, and list literals.

```text
42, 3.14, "hello", true, null, [1, 2, 3]
```

### [Column References](variables.md#column-references)

Use `@` prefix to reference columns.

```text
@1, @col_name, @"col name"
```

### [Variable Binding](variables.md#variable-binding)

Use `as` to bind values to variables.

```text
@price * @qty as @total; @total * 1.1
```

### [Operators](operators.md)

Arithmetic, comparison, logical, and pipe operators.

```text
+ - * / %, == != < >, and or, |
```

### [Function Calls](syntax.md#function-calls)

Prefix calls, pipe calls, and method calls.

```text
trim(@name)
@name | trim() | upper()
@name.trim().upper()
```

### Documentation Index

- [Literals](expr/literals.md) - Literal syntax and type system
- [Variables](expr/variables.md) - Column references and variable binding
- [Operators](expr/operators.md) - Operator precedence and details
- [Functions](expr/functions.md) - Complete function reference
- [Syntax Guide](expr/syntax.md) - Complete syntax documentation
- [Rosetta Code](expr/rosetta.md) - Fun programs

### Expr Commands

Comparing modes and other commands:

| Command              | What it does        | Input row         | Output row        |
|----------------------|---------------------|-------------------|-------------------|
| `expr`               | Evaluate to new row | `a, b`            | `c`               |
| `expr -m add`        | Add new column(s)   | `a, b`            | `a, b, c`         |
| `expr -m mutate`     | Modify column value | `a, b`            | `a, c`            |
| `expr -m filter`     | Keep or discard row | `a, b`            | `a, b` or nothing |
| `filter`             |                     | `a, b`            | `a, b` or nothing |
| `expr -E '[@b, @c]'` | Select columns      | `a, b, c`         | `b, c`            |
| `select`             |                     | `a, b, c`         | `b, c`            |
| `join`               | Join two tables     | `a, b` and `a, c` | `a, b, c`         |

Note: Use `tva filter`/`tva select` for simple tasks: they are ~2x faster. Use `tva expr`
only when you need advanced features that other commands don't support (functions, complex
expressions, etc.).

## Basic Usage

### `tva expr` - Evaluate Expressions to New Row

Evaluates expression for each row and outputs the expression result as a new row (original row
structure is discarded).

```bash
# Simple arithmetic expression
tva expr -E "10 + 20"

# Evaluate expression with row data
tva expr -n "price,qty" -r "100,2" -E "@price * @qty"

# String manipulation
tva expr -n "name" -r "  alice  " -E 'upper(trim(@name))'
tva expr -n "name" -r "  alice  " -E '@name.trim().upper()'
tva expr -n "name" -r "  alice  " -E '@name | trim() | upper()'

# Header - @price / @carat
tva expr -H -E "@price / @carat" docs/data/diamonds.tsv | tva slice -r -5

# Header - price_per_carat carat
tva expr -H -E "[@price / @carat as @price_per_carat, @carat]" docs/data/diamonds.tsv | tva slice -r -5

# Add new column
tva expr -H -m add -E "@price / @carat as @price_per_carat" docs/data/diamonds.tsv | tva slice -r -5

tva expr -H -m add -E "[@price / @carat as @price_per_carat, @carat]" docs/data/diamonds.tsv | tva slice -r -5

tva expr -H -m add -E "[@price / @carat as @price_per_carat, @carat as @carat_rounded]" docs/data/diamonds.tsv | tva slice -r -5

# Filter rows using -m skip-null
tva expr -H -m s -E 'if(@carat > 1 and @cut eq q(Premium) and @price < 3000, @0, null)' docs/data/diamonds.tsv

# Filter rows using -m filter
tva expr -H -m f -E '@carat > 1 and @cut eq q(Premium) and @price < 3000' docs/data/diamonds.tsv

```

## Notes

- No implicit type conversion - use explicit functions like `int()`, `float()`, `string()`
- String comparison uses `eq`, `ne`, `lt`, etc. (not `==`, `!=`)
- Pipe operator `|` passes left value as first argument to right function
- All expressions are evaluated per row during streaming
