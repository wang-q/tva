# Expression Engine

TVA provides a powerful expression engine for data transformation and filtering.

## Quick Reference

- [Expression Syntax](expr-syntax.md) - Complete syntax guide
- [Functions & Operators](expr/functions.md) - Function and operator reference

## Overview

The expression engine supports:

- **Column references**: `@column_name` or `@1` (1-based index)
- **Literals**: integers, floats, strings, booleans, null, lists
- **Operators**: arithmetic, comparison, logical, string, pipe
- **Functions**: 40+ built-in functions for string, numeric, list operations
- **Lambda expressions**: `x => x * 2` for higher-order functions

### Four Expression Commands

| Command | Description |
|---------|-------------|
| `tva expr` | Evaluate expression to create a new row |
| `tva filter` | Filter rows based on condition |
| `tva map` | Add new column(s) to existing row |
| `tva apply` | Update existing column value |

## Basic Usage

### `tva expr` - Evaluate Expression to New Row

Evaluates expression for each row and outputs the expression result as a new row (original row structure is discarded).

```bash
# Evaluate expression to create new row
tva expr -E "@price * @qty as @total" data.tsv

# Multiple expressions with semicolon
tva expr -E '@price | float() as @p; @qty | int() as @q; @p * @q as @total' data.tsv

# String manipulation
tva expr -E '@name | trim() | upper() as @clean_name' data.tsv

# Conditional expression
tva expr -E 'if(@score >= 60, "pass", "fail") as @result' data.tsv
```

### `tva filter` - Filter Rows

Evaluates expression for each row to decide whether to output the row.

```bash
# Filter rows where age > 18
tva filter -E "@age > 18" data.tsv

# Filter with string comparison
tva filter -E '@status eq "active"' data.tsv

# Complex conditions
tva filter -E '@age > 18 and @status eq "active"' data.tsv
```

### `tva map` - Add New Column

Evaluates expression for each row to add a new column to the existing row.

```bash
# Add a new column with calculated value
tva map -E "@price * @qty as @total" data.tsv

# Add multiple columns
tva map -E '@price * 1.1 as @price_with_tax, @qty * 2 as @double_qty' data.tsv

# Add column with transformed value
tva map -E '@email | lower() | trim() as @clean_email' data.tsv
```

### `tva apply` - Update Column

Evaluates expression for each row to update an existing column value.

```bash
# Modify existing column
tva apply -E '@price | float() * 1.1' -c price data.tsv

# Transform column value
tva apply -E '@name | trim() | upper()' -c name data.tsv

# Replace with conditional value
tva apply -E 'if(@score >= 60, "PASS", "FAIL")' -c status data.tsv
```

## Examples

### Numeric Operations

```rust
@price * 1.1                    # Increase price by 10%
(@price - @cost) / @cost        # Calculate margin
round(@value)                   # Round to integer
```

### String Operations

```rust
@first ++ " " ++ @last          # Concatenate names
@email | lower() | trim()       # Chain operations
@name | substr(_, 0, 5)         # First 5 characters
split(@tags, ",")               # Split to list
```

### List Operations

*Note: `map()` and `filter()` here are **functions**, not the `tva map` and `tva filter` commands.*

```rust
map([1, 2, 3], x => x * 2)                    # [2, 4, 6]
filter([1, 2, 3, 4], x => x > 2)              # [3, 4]
reduce([1, 2, 3], 0, (acc, x) => acc + x)     # 6
join(split(@tags, ","), "; ")                  # Rejoin with different separator
```

### Conditional Logic

```rust
if(@age >= 18, "adult", "minor")
default(@nickname, @username)    # Use nickname if not empty
```

## Type System

| Type | Example | Notes |
|------|---------|-------|
| Int | `42`, `-10` | 64-bit signed |
| Float | `3.14`, `-0.5` | Double precision |
| String | `"hello"` | UTF-8 encoded |
| Bool | `true`, `false` | |
| Null | `null` | Missing/empty value |
| List | `[1, 2, 3]` | Heterogeneous elements |

## Command Comparison

| Command | What it does | Input row | Output row | Columns changed |
|---------|--------------|-----------|------------|-----------------|
| `expr` | Evaluate to new row | `@a, @b, @c` | `@result` | All (replaced) |
| `filter` | Keep or discard row | `@a, @b, @c` | `@a, @b, @c` or nothing | None |
| `map` | Add new column(s) | `@a, @b` | `@a, @b, @c` | Added |
| `apply` | Update column value | `@a, @b, @c` | `@a, @b', @c` | One updated |

## Notes

- No implicit type conversion - use explicit functions like `int()`, `float()`, `string()`
- String comparison uses `eq`, `ne`, `lt`, etc. (not `==`, `!=`)
- Pipe operator `|` passes left value as first argument to right function
- All expressions are evaluated per row during streaming
