# String Formatting (fmt)

The `fmt()` function provides powerful string formatting capabilities, inspired by Rust's `format!` macro and Perl's `q//` operator.

## Overview

```bash
fmt(template: string, ...args: any) -> string
```

The `fmt` function uses `%` as the prefix for placeholders and supports three types of delimiters to avoid conflicts with different content:

- `%(...)` - Parentheses (default)
- `%[...]` - Square brackets
- `%{...}` - Curly braces

## Placeholder Forms

| Form | Description | Example |
|:-----|:------------|:--------|
| `%()` | Next positional argument | `fmt("%() %()", a, b)` |
| `%(n)` | nth positional argument (1-based) | `fmt("%(2) %(1)", a, b)` |
| `%(var)` | Lambda parameter reference | `fmt("%(name)")` |
| `%(@n)` | Column by index | `fmt("%(@1) and %(@2)")` |
| `%(@var)` | Variable reference | `fmt("%(@name)")` |

## Format Specifiers

Format specifiers follow the colon `:` after the placeholder content:

```
%(placeholder:format_spec)
```

### Fill and Align

| Align | Description | Example `%(:*<10)` |
|:------|:------------|:-------------------|
| `<` | Left align | `hello*****` |
| `>` | Right align | `*****hello` |
| `^` | Center | `**hello***` |

### Sign

| Sign | Description | Example |
|:-----|:------------|:--------|
| `-` | Only negative (default) | `-42` |
| `+` | Always show sign | `+42`, `-42` |

### Alternative Form (`#`)

| Type | Effect | Example `%(:#x)` |
|:-----|:-------|:-----------------|
| `x` | Add `0x` prefix | `0xff` |
| `X` | Add `0X` prefix | `0XFF` |
| `b` | Add `0b` prefix | `0b1010` |
| `o` | Add `0o` prefix | `0o77` |

### Width and Precision

- **Width**: Minimum field width
- **Precision**: For integers - zero pad; for floats - decimal places; for strings - max length

### Type Specifiers

| Type | Description | Example |
|:-----|:------------|:--------|
| (omit) | Default | Auto-select by type |
| `b` | Binary | `1010` |
| `o` | Octal | `77` |
| `x` / `X` | Hexadecimal | `ff` / `FF` |
| `e` / `E` | Scientific notation | `1.23e+04` |

## Basic Examples

```bash
# Basic formatting
tva expr -E 'fmt("Hello, %()!", "world")'           # "Hello, world!"
tva expr -E 'fmt("%() + %() = %()", 1, 2, 3)'        # "1 + 2 = 3"

# Position arguments (1-based)
tva expr -E 'fmt("%(2) %(1)", "world", "Hello")'    # "Hello world"

# Format specifiers
tva expr -E 'fmt("%(:>10)", "hi")'                  # "        hi"
tva expr -E 'fmt("%(:*<10)", "hi")'                 # "hi********"
tva expr -E 'fmt("%(:^10)", "hi")'                  # "    hi    "

# Number formatting
tva expr -E 'fmt("%(:+)", 42)'                      # "+42"
tva expr -E 'fmt("%(:08)", 42)'                     # "00000042"
tva expr -E 'fmt("%(:.2)", 3.14159)'                # "3.14"

# Number bases
tva expr -E 'fmt("%(:b)", 42)'                      # "101010"
tva expr -E 'fmt("%(:x)", 255)'                     # "ff"
tva expr -E 'fmt("%(:#x)", 255)'                    # "0xff"

# String truncation
tva expr -E 'fmt("%(:.5)", "hello world")'          # "hello"
```

## Column References

Use `%(@n)` to reference columns directly without passing them as arguments:

```bash
# Reference columns by index
tva expr -E 'fmt("%(@1) has %(@2) points")' -r "Alice,100"
# Output: Alice has 100 points

# With format specifiers (note: column values are treated as strings by default)
tva expr -E 'fmt("%(@1): %(@2) points")' -r "Alice,100"
# Output: Alice: 100 points

tva expr -E 'fmt("%(): %(@2) points", @1)' -r "Alice,100"

```

## Lambda Variables

Reference lambda parameters within `fmt`:

```bash
# Using %(var) in lambda
tva expr -E 'map([1, 2, 3], x => fmt("value: %(x)"))'
# Output: value: 1    value: 2    value: 3

# Using %[var] to avoid conflicts
tva expr -E 'map([1, 2, 3], x => fmt(q(value: %[x])))'
# Output: value: 1    value: 2    value: 3
```

## Variable References

Use `%(@var)` to reference variables defined with `as @var`:

```bash
# Basic variable reference
tva expr -E '
    "Bob" as @name;
    fmt("Hello, %(@name)!")
'
# Output: Hello, Bob!

# Variable with format specifier
tva expr -E '
    3.14159 as @pi;
    fmt("Pi = %(@pi:.2)")
'
# Output: Pi = 3.14

# Multiple variables
tva expr -E '
    42 as @num;
    fmt("Hex: %(@num:#x), Bin: %(@num:b)")
'
# Output: Hex: 0x2a, Bin: 101010

# Using with -r option and global variables
tva expr -r "Alice,100" -r "Bob,200" -E '
    fmt("Hello, %(@1)! from line %(@__index)")
'
# Output: Hello, Alice! from line 1
#         Hello, Bob! from line 2

# Accumulating values across rows
tva expr -r "Alice,100" -r "Bob,200" -E '
    default(@__sum, 0) + @2 as @__sum;
    fmt("Hello, %(@1)! sum: %(@__sum)")
'
# Output: Hello, Alice! sum: 100
#         Hello, Bob! sum: 300
```

## Delimiter Selection

Choose different delimiters to avoid conflicts with your content:

```bash
# Use %[] when template contains ()
tva expr -E 'fmt("Result: %[:.2]", 3.14159)'
# Output: Result: 3.14

# Use %{} when template contains []
tva expr -E 'fmt("%{1:+}", 42)'
# Output: +42

# Using q() with %[] to avoid escaping quotes
tva expr -E 'fmt(q(The "value" is %[1]), 42)'
# Output: The "value" is 42
```

**Note:** `q()` strings cannot contain unescaped `(` or `)`. Use `%[]` or `%{}` instead.

## Using with GNU Parallel

The `%()` syntax doesn't conflict with GNU parallel's `{}`:

```bash
# Safe to use together
parallel 'tva expr -E "fmt(q(Processing: %[] at %[]), {}, now())"' ::: *.tsv

# Format file names
parallel 'tva expr -E '"'"'fmt("File: %(1)", {})'"'"'' ::: *.txt
```

## Comparison with Rust format!

| Feature | Rust | tva fmt |
|:--------|:-----|:--------|
| Placeholder | `{}` | `%()` / `%[]` / `%{}` |
| Position index | 0-based | 1-based |
| Named parameters | `format!("{name}", name="val")` | Use `%(var)` with lambda |
| Dynamic width | `format!("{:>1$}", x, width)` | Not supported |
| Dynamic precision | `format!("{:.1$}", x, prec)` | Not supported |
| Debug format (`?`) | `{:?}` | Not supported |
| Argument counting | Compile-time check | Runtime check |

## Escape Sequences

Use `%%` to output a literal percent sign:

```bash
tva expr -E 'fmt("100%% complete")'   # "100% complete"
```
