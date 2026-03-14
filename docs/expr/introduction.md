# Expression Introduction

TVA Expression is a concise data processing language for `tva expr`, `tva filter`, and `tva mutate` commands.

## Quick Examples

```bash
# Basic arithmetic
tva expr -E '42 + 3.14'

# Filter records
tva filter -E "@price > 100" data.tsv

# Compute new column
tva mutate -E "@price * 1.1 as @price_with_tax" data.tsv
```

## Topics

### [Literals](literals.md)

Integer, float, string, boolean, null, and list literals.

```rust
42, 3.14, "hello", true, null, [1, 2, 3]
```

### [Column References](variables.md#column-references)

Use `@` prefix to reference columns.

```rust
@1, @col_name, @"col name"
```

### [Variable Binding](variables.md#variable-binding)

Use `as` to bind values to variables.

```rust
@price * @qty as @total; @total * 1.1
```

### [Operators](operators.md)

Arithmetic, comparison, logical, and pipe operators.

```rust
+ - * / %, == != < >, and or, |
```

### [Function Calls](expressions.md)

Prefix calls, pipe calls, and method calls.

```rust
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
