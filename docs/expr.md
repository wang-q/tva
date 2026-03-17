# TVA's expr language and companions

The `expr` language evaluates expressions (like spreadsheet formulas) to transform TSV data.
It powers three commands: `tva expr` (create new rows), `tva map` (add columns), and
`tva mutate` (modify columns).

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

| Command      | Description                             |
|--------------|-----------------------------------------|
| `tva expr`   | Evaluate expression to create a new row |
| `tva map`    | Add new column(s) to existing row       |
| `tva mutate` | Modify existing column value            |

| Command  | What it does        | Input row | Output row          | Columns changed |
|----------|---------------------|-----------|---------------------|-----------------|
| `expr`   | Evaluate to new row | `@a, @b`  | `@c`                | All (replaced)  |
| `map`    | Add new column(s)   | `@a, @b`  | `@a, @b, @c`        | Added           |
| `mutate` | Modify column value | `@a, @b`  | `@a, @c`            | One updated     |
| `filter` | Keep or discard row | `@a, @b`  | `@a, @b` or nothing | None            |

Note: Use `tva filter` for simple filtering—it's ~2x faster. Use `tva expr --skip-null`
only when you need features `tva filter` doesn't support (functions, complex expressions, etc.).

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

# Conditional expression
tva expr -n "score" -r "85" -E 'if(@score >= 60, "pass", "fail")'

# Process TSV file - calculate price per carat
tva expr -H -E "@price / @carat" docs/data/diamonds.tsv | tva slice -r -5

# Filter rows using --skip-null
tva expr -H --skip-null -E 'if(@carat > 1 and @cut eq q(Premium) and @price < 3000, @0, null)' docs/data/diamonds.tsv
```

### `tva map` - Add New Column

Evaluates expression for each row to add a new column to the existing row.

```bash
# Add a new column with calculated value
tva map -E "@price * 1.1 as @price_with_tax" docs/data/diamonds.tsv

# Add multiple columns
tva map -E '@price * 1.1 as @price_with_tax, @carat * 2 as @double_carat' docs/data/diamonds.tsv

# Add column with transformed value
tva map -E '@cut | lower() as @cut_lower' docs/data/diamonds.tsv
```

### `tva mutate` - Modify Column

Evaluates expression for each row to modify an existing column value.

```bash
# Modify existing column
tva mutate -E '@price * 1.1' -c price docs/data/diamonds.tsv

# Transform column value
tva mutate -E '@cut | upper()' -c cut docs/data/diamonds.tsv

# Replace with conditional value
tva mutate -E 'if(@price >= 350, "expensive", "cheap")' -c price docs/data/diamonds.tsv
```

## Notes

- No implicit type conversion - use explicit functions like `int()`, `float()`, `string()`
- String comparison uses `eq`, `ne`, `lt`, etc. (not `==`, `!=`)
- Pipe operator `|` passes left value as first argument to right function
- All expressions are evaluated per row during streaming
