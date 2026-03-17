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

# Using higher-order functions (list results expand to multiple columns)
tva expr -E "map([1,2,3,4,5], x => x * x)"
# Output: 1       4       9       16      25
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

| Command                   | What it does        | Input row         | Output row        |
|---------------------------|---------------------|-------------------|-------------------|
| `expr`/`expr -m eval`     | Evaluate to new row | `a, b`            | `c`               |
| `extend`/`expr -m extend` | Add new column(s)   | `a, b`            | `a, b, c`         |
| `mutate`/`expr -m mutate` | Modify column value | `a, b`            | `a, c`            |
| `expr -m skip-null`       | Skip null results   | `a, b`            | `c` or nothing    |
| `expr -m filter`          | Keep or discard row | `a, b`            | `a, b` or nothing |
| `filter`                  |                     | `a, b`            | `a, b` or nothing |
| `expr -E '[@b, @c]'`      | Select columns      | `a, b, c`         | `b, c`            |
| `select`                  |                     | `a, b, c`         | `b, c`            |
| `join`                    | Join two tables     | `a, b` and `a, c` | `a, b, c`         |

## Output Modes

The `expr` command supports five output modes controlled by the `-m` (or `--mode`) flag:

### `eval` mode (default, `-m eval` or `-m e`)

Evaluates the expression and outputs only the result. The original row data is discarded.

```bash
# Simple arithmetic expression (no input needed)
tva expr -E "10 + 20"

# Evaluate expression with inline row data
tva expr -n "price,qty" -r "100,2" -E "@price * @qty"

# String manipulation with inline data
tva expr -n "name" -r "  alice  " -E '@name | trim() | upper()'

# Calculate from file data
tva expr -H -E "@price / @carat" docs/data/diamonds.tsv | tva slice -r 5
```

Use this mode when you want to compute new values without preserving the original columns.

### `extend` mode (`-m extend` or `-m a`)

Evaluates the expression and appends the result as new column(s) to the original row.

```bash
# Add a single column
tva expr -H -m extend -E "@price / @carat as @price_per_carat" docs/data/diamonds.tsv | tva slice -r 5

# Add multiple columns using list expression
tva expr -H -m extend -E "[@price / @carat as @price_per_carat, @carat as @carat_rounded]" docs/data/diamonds.tsv | tva slice -r 5
```

Key behaviors:

- The original row is preserved
- Expression results are appended as new columns
- Header names come from `as @name` bindings
- List expressions create multiple new columns

### `mutate` mode (`-m mutate` or `-m u`)

Modifies an existing column in place. The expression must include an `as @column_name` binding to
specify which column to modify.

```bash
# Modify price column in place
tva expr -H -m mutate -E "@price / @carat as @price" docs/data/diamonds.tsv | tva slice -r 5
```

Key behaviors:

- Only the specified column is modified
- All other columns and the header remain unchanged
- The `as @column_name` binding is required
- Column name must exist in the input (numeric indices like `as @2` are not supported)

### `skip-null` mode (`-m skip-null` or `-m s`)

Evaluates the expression and outputs the result, but skips rows where the result is `null`.

```bash
# Keep rows where carat > 1 and cut is Premium and price < 3000
tva expr -H -m skip-null -E 'if(@carat > 1 and @cut eq q(Premium) and @price < 3000, @0, null)' docs/data/diamonds.tsv | tva slice -r 5
```

Key behaviors:

- Rows with null results are excluded from output
- Useful for filtering based on complex conditions
- Return `@0` to preserve the original row, or any other value to output that value

### `filter` mode (`-m filter` or `-m f`)

Evaluates a boolean expression and outputs the original row only when the expression is true.

```bash
# Filter with a simple condition
tva expr -H -m filter -E "@price > 10000" docs/data/diamonds.tsv | tva slice -r 5

# Filter with multiple conditions
tva expr -H -m filter -E '@carat > 1 and @cut eq q(Premium) and @price < 3000' docs/data/diamonds.tsv | tva slice -r 5
```

Key behaviors:

- The original row and header are preserved
- Row is output only if the expression evaluates to true
- Expression should return a boolean (non-zero numbers and non-empty strings are truthy)
- Similar to `tva filter` but allows complex expressions

## Notes

- **Performance**: For simple filtering or column selection, use `tva filter` or `tva select`
  instead - they are ~2x faster. Use `tva expr` only when you need functions, complex expressions,
  or calculations.
- **Type conversion**: No implicit type conversion - use explicit functions like `int()`, `float()`,
  `string()`
- **String comparison**: Uses `eq`, `ne`, `lt`, etc. (not `==`, `!=`)
- **Pipe operator**: `|` passes left value as first argument to right function
- **Streaming**: All expressions are evaluated per row during streaming
- **Persistent variables**: Variables starting with `__` (e.g., `@__total`) persist across rows,
  useful for running totals
