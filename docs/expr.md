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

- [Literals](literals.md) - Literal syntax and type system
- [Variables](variables.md) - Column references and variable binding
- [Operators](operators.md) - Operator precedence and details
- [Functions](functions.md) - Complete function reference
- [Syntax Guide](expr/syntax.md) - Complete syntax documentation
- [Rosetta Code](expr/rosetta.md) - Fun programs

- **Lambda expressions**: `x => x * 2` for higher-order functions

### Expression Commands

| Command      | Description                             |
|--------------|-----------------------------------------|
| `tva expr`   | Evaluate expression to create a new row |
| `tva map`    | Add new column(s) to existing row       |
| `tva mutate` | Modify existing column value            |

Note: Use `tva filter` for simple filtering—it's ~2x faster. Use `tva expr --skip-null`
only when you need features `tva filter` doesn't support (functions, complex expressions, etc.).

## Basic Usage

### `tva expr` - Evaluate Expression to New Row

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
tva expr -H -E "@price / @carat" docs/data/diamonds.tsv

# Filter rows using --skip-null
tva expr -H --skip-null -E 'if(@price > 300, @0, null)' docs/data/diamonds.tsv
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

## Examples

### Numeric Operations

```rust
@price * 1.1 # Increase price by 10 %
( @ price - @ cost) / @ cost # Calculate margin
round( @ value) # Round to integer
```

### String Operations

```rust
@first + + " " + + @ last # Concatenate names
@email | lower() | trim() # Chain operations
@name | substr(_, 0, 5) # First 5 characters
split( @ tags, ",") # Split to list
```

### List Operations

The `map()` and `filter()` below are **functions** for working with lists, not the `tva map`
and `tva filter` commands.

```rust
map([1, 2, 3], x => x * 2)                    #[2, 4, 6]
filter([1, 2, 3, 4], x => x > 2)              #[3, 4]
reduce([1, 2, 3], 0, (acc, x) => acc + x) # 6
join(split( @ tags, ","), "; ") # Rejoin with different separator
```

### Conditional Logic

```rust
if( @ age > = 18, "adult", "minor")
default ( @ nickname, @ username) # Use nickname if not empty
```

## Type System

| Type   | Example         | Notes                  |
|--------|-----------------|------------------------|
| Int    | `42`, `-10`     | 64-bit signed          |
| Float  | `3.14`, `-0.5`  | Double precision       |
| String | `"hello"`       | UTF-8 encoded          |
| Bool   | `true`, `false` |                        |
| Null   | `null`          | Missing/empty value    |
| List   | `[1, 2, 3]`     | Heterogeneous elements |

## Command Comparison

| Command  | What it does        | Input row    | Output row              | Columns changed |
|----------|---------------------|--------------|-------------------------|-----------------|
| `expr`   | Evaluate to new row | `@a, @b, @c` | `@result`               | All (replaced)  |
| `filter` | Keep or discard row | `@a, @b, @c` | `@a, @b, @c` or nothing | None            |
| `map`    | Add new column(s)   | `@a, @b`     | `@a, @b, @c`            | Added           |
| `mutate` | Modify column value | `@a, @b, @c` | `@a, @b', @c`           | One updated     |

## Performance Notes

The expression engine includes several optimizations for better performance:

* **Parse caching**: Expressions are parsed once and cached for all rows. Identical expressions reuse the cached AST.
* **Column name resolution**: When headers are available, `@name` references are resolved to `@index` at parse time for O(1) access.
* **Constant folding**: Constant sub-expressions (e.g., `2 + 3 * 4`) are pre-computed during parsing.
* **Function registry**: Built-in functions are looked up once and cached, avoiding repeated hash map lookups.
* **Hash algorithm**: Uses `ahash` for faster hash map operations.

For best performance, use column indices (`@1`, `@2`) instead of names.

## Notes

- No implicit type conversion - use explicit functions like `int()`, `float()`, `string()`
- String comparison uses `eq`, `ne`, `lt`, etc. (not `==`, `!=`)
- Pipe operator `|` passes left value as first argument to right function
- All expressions are evaluated per row during streaming
