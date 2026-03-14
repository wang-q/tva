# Expression Language

TVA provides a powerful expression language for data transformation and filtering.

## Quick Reference

- [Introduction](expr/introduction.md) - Expression language overview
- [Syntax Guide](expr/expressions.md) - Complete syntax documentation
- [Functions](expr/functions.md) - Function reference
- [Rosetta Code](expr/rosetta.md) - Example programs

## Overview

The expression language supports:

- **Column references**: `@column_name` or `@1` (1-based index)
- **Literals**: integers, floats, strings, booleans, null, lists
- **Operators**: arithmetic, comparison, logical, string, pipe
- **Variable binding**: `expr as @var` for reusing results
- **Functions**: 40+ built-in functions
- **Lambda expressions**: `x => x * 2` for higher-order functions

### Four Expression Commands

| Command      | Description                             |
|--------------|-----------------------------------------|
| `tva expr`   | Evaluate expression to create a new row |
| `tva filter` | Filter rows based on condition          |
| `tva map`    | Add new column(s) to existing row       |
| `tva apply`  | Update existing column value            |

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
```

### `tva filter` - Filter Rows

Evaluates expression for each row to decide whether to output the row.

```bash
# Filter rows where price > 300
tva filter -E "@price > 300" docs/data/diamonds.tsv

# Filter with string comparison
tva filter -E '@cut eq "Ideal"' docs/data/diamonds.tsv

# Complex conditions
tva filter -E '@price > 300 and @cut eq "Ideal"' docs/data/diamonds.tsv
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

### `tva apply` - Update Column

Evaluates expression for each row to update an existing column value.

```bash
# Modify existing column
tva apply -E '@price * 1.1' -c price docs/data/diamonds.tsv

# Transform column value
tva apply -E '@cut | upper()' -c cut docs/data/diamonds.tsv

# Replace with conditional value
tva apply -E 'if(@price >= 350, "expensive", "cheap")' -c price docs/data/diamonds.tsv
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

*Note: `map()` and `filter()` here are **functions**, not the `tva map` and `tva filter` commands.*

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
| `apply`  | Update column value | `@a, @b, @c` | `@a, @b', @c`           | One updated     |

## Notes

- No implicit type conversion - use explicit functions like `int()`, `float()`, `string()`
- String comparison uses `eq`, `ne`, `lt`, etc. (not `==`, `!=`)
- Pipe operator `|` passes left value as first argument to right function
- All expressions are evaluated per row during streaming
