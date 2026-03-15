# Expression Introduction

TVA Expression is a concise data processing language designed for `tva expr`, `tva filter`, and `tva mutate`
commands. It provides a familiar syntax inspired by JavaScript, Python, and jq, optimized for TSV data processing.

## Quick Examples

```bash
# Basic arithmetic
tva expr -E '42 + 3.14'
# Output: 45.14

# String manipulation
tva expr -E '"hello" | upper()'
# Output: HELLO

# Filter records
tva filter -E "@price > 100" data.tsv

# Compute new column
tva mutate -E "@price * 1.1 as @price_with_tax" data.tsv
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

### [Function Calls](expressions.md)

Prefix calls, pipe calls, and method calls.

```text
trim(@name)
@name | trim() | upper()
@name.trim().upper()
```

## Documentation Index

- [Literals](literals.md) - Literal syntax and type system
- [Variables](variables.md) - Column references and variable binding
- [Expressions](expressions.md) - Function calls, pipelines, multiple expressions
- [Operators](operators.md) - Operator precedence and details
- [Functions](functions.md) - Complete function reference

## Design Philosophy

TVA Expression is designed with these principles:

1. **Shell-friendly**: `@` prefix avoids conflicts with shell variables
2. **Familiar syntax**: Inspired by popular languages (JavaScript, Python, jq)
3. **Type coercion**: Automatic conversion between types where sensible
4. **Null safety**: Empty TSV fields are treated as `null`, not empty strings
5. **Composability**: Pipe operator enables clean data transformation chains
