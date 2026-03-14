# Expression Syntax Guide

This document provides a comprehensive guide to TVA expression syntax, covering function calls, pipelines, and multi-expression evaluation.

## Function Call Syntax

### Prefix Call

`func(arg1, arg2, ...)` - Traditional function call syntax.

```bash
tva expr -E 'trim("  hello  ")'             # Returns: hello
tva expr -E 'substr("hello world", 0, 5)'   # Returns: hello
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

## See Also

- [Literals and Variables](literals.md) - Literal syntax, type system, column references
- [Functions](functions.md) - Complete function reference
