# Expression Syntax Guide

This document provides a comprehensive guide to TVA expression syntax, covering function calls, pipelines, and multi-expression evaluation.

## Expression Elements

TVA expressions are composed of the following atomic elements:

| Element | Syntax | Description |
| :--- | :--- | :--- |
| **Column Reference** | `@1`, `@col_name` | Reference input data columns |
| **Variable** | `@var_name` | Variables bound via `as` |
| **Literal** | `42`, `"hello"`, `true`, `null`, `[1, 2, 3]` | Constant values |
| **Function Call** | `func(args...)` | Built-in functions |

## Evaluation Rules

*   Expressions are evaluated left-to-right according to operator precedence
*   The pipe operator `|` has the lowest precedence, used to connect multiple processing steps

## Function Call Syntax

### Prefix Call

`func(arg1, arg2, ...)` - Traditional function call syntax.

```bash
tva expr -E 'trim("  hello  ")'             # Returns: hello
tva expr -E 'substr("hello world", 0, 5)'   # Returns: hello
```

### Method Call

Method call is syntactic sugar for function calls:

```bash
# Method call is equivalent to function call
@name.trim()           # Equivalent to: trim(@name)
@price.round()         # Equivalent to: round(@price)

# Method chaining
@name.trim().upper().substr(0, 5)
# Equivalent to: substr(upper(trim(@name)), 0, 5)

# Method call with arguments
@name.substr(0, 5)     # Equivalent to: substr(@name, 0, 5)
@price.pow(2)          # Equivalent to: pow(@price, 2)
```

### Pipe Call (Single Argument)

`arg | func()` - Pipe left value to function. The `_` placeholder can be omitted for single-argument functions.

```bash
tva expr -E '"hello" | upper()'             # Returns: HELLO
tva expr -E '[1, 2, 3] | reverse()'         # Returns: [3, 2, 1]
```

### Pipe Call (Multiple Arguments)

`arg | func(_, arg2)` - Use `_` to represent the piped value as the first argument.

```bash
tva expr -E '"hello world" | substr(_, 0, 5)'   # Returns: hello
tva expr -E '"a,b,c" | split(_, ",")'           # Returns: ["a", "b", "c"]
```

## Expression Composition

Expressions can be combined in several ways:

*   **Operator Composition**: `@a + @b`, `@x > 10 and @y < 20`
*   **Pipe Composition**: `@name | trim() | upper()`
*   **Variable Binding**: `expr as @var; @var + 1`
*   **Function Nesting**: `if(@age > 18, "adult", "minor")`

## Lambda Expressions

Lambda expressions create anonymous functions, primarily used with higher-order functions like `map`, `filter`, and `reduce`:

### Syntax

| Form | Syntax | Example |
| :--- | :--- | :--- |
| Single parameter | `param => expr` | `x => x + 1` |
| Multiple parameters | `(p1, p2, ...) => expr` | `(x, y) => x + y` |
| No parameters | `() => expr` | `() => 42` |

### Examples

```bash
# Single-parameter lambda
tva expr -E 'map([1, 2, 3], x => x * 2)'
# Returns: [2, 4, 6]

# Multi-parameter lambda
tva expr -E 'reduce([1, 2, 3], 0, (acc, x) => acc + x)'
# Returns: 6

# Filter with lambda
tva expr -E 'filter([1, 2, 3, 4], x => x > 2)'
# Returns: [3, 4]
```

Lambda bodies can reference columns (`@col`) and variables (`@var`) from the outer scope.

## Complex Pipelines

The pipe operator `|` enables powerful function chaining:

```bash
# Chain single-argument functions
tva expr -n "name" -r "  john doe  " -E '@name | trim() | upper()'
# Returns: JOHN DOE

# Mix single and multi-argument functions
tva expr -n "desc" -r "hello world" -E '@desc | substr(_, 0, 5) | upper()'
# Returns: HELLO

# Complex validation pipeline
tva expr -n "email" -r "  Test@Example.COM  " -E '@email | trim() | lower() | regex_match(_, ".*@.*\\.com")'
# Returns: true
```

## Multiple Expressions

Use `;` to separate multiple expressions, evaluated sequentially:

```bash
# Multiple expressions with variable binding
tva expr -n "price,qty" -r "10,5" -E '@price as @p; @qty as @q; @p * @q'
# Returns: 50

# Pipeline and semicolons
tva expr -n "price,qty" -r "10,5" -E '
    @price | int() as @p;
    @p * 2 as @p;
    @qty | int() as @q;
    @q * 3 as @q;
    @p + @q
'
# Returns: 35
```

**Rules**:
- Each expression can have side effects (like variable binding)
- Only the last expression's value is returned

## Comments

TVA supports line comments starting with `//`. Comments are only valid inside expressions; comments in command line are handled by the Shell.

```bash
# With comments explaining the logic
tva expr -n "total,tax" -r "100,0.1" -E '
    @total | int() as @t;  // Convert to integer
    @tax | float() as @r;  // Convert tax rate to float
    @t * (1 + @r)          // Calculate total with tax
'
# Returns: 110

tva expr -n "price,qty,tax_rate" -r "10,5,0.1" -E '
    // Calculate total price
    @price * @qty as @total;
    @total * (1 + @tax_rate)  // With tax
'
# Returns: 55
```

## Output Behavior

In `tva expr`, the last expression's value is printed to stdout:

```bash
# Simple expression output
tva expr -E '42 + 3.14'           # Prints: 45.14

# Column reference output
tva expr -n "name" -r "John" -E '@name'   # Prints: John
```

The `print(val, ...)` function outputs multiple arguments sequentially and returns the last argument's value. If `print()` is the last expression, the value won't be printed twice:

```bash
# Print intermediate values
tva expr -n "price,qty" -r "10,5" -E '
    @price | print("price:", _);
    print("qty:", @qty);
    @price * @qty
'
# Prints: price: 10
#         qty: 5
#         50
```
