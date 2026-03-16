# Expr Variables

TVA expressions support two kinds of `@`-prefixed identifiers: column references and variables.

## Column References

Use `@` prefix to reference columns, avoiding conflicts with Shell variables:

| Syntax | Description | Example |
|:-------|:------------|:--------|
| `@0` | Entire row content (all columns joined with tabs) | `@0` |
| `@1`, `@2` | 1-based column index | `@1` is the first column |
| `@col_name` | Column name reference | `@price` references the price column |
| `@"col name"` or `@'col name'` | Column name with spaces | `@"user name"` references column "user name" |

**Design rationale**:

- **Shell-friendly**: `@` has no special meaning in bash/zsh, no escaping needed
- **Concise**: Only 2 characters (`Shift+2`)

### Type Behavior

- Column references return `String` by default (raw bytes from TSV)
- Numeric operations automatically attempt parsing; failure yields `null`
- Use `int(@col)` or `float(@col)` for explicit type specification
- **Empty fields are treated as `null`**, not empty strings. See [Null Type and Empty Fields](literals.md#null-type-and-empty-fields) for details.

```bash
# Column by index
tva expr -n "name,age" -r "John,30" -E '@1'       # Returns: John
tva expr -n "name,age" -r "John,30" -E '@2'       # Returns: 30 (parsed as int)

# Column by name
tva expr -n "name,age" -r "John,30" -E '@name'    # Returns: John
tva expr -n "name,age" -r "John,30" -E '@age'     # Returns: 30

# Entire row
tva expr -n "a,b,c" -r "1,2,3" -E '@0'            # Returns: "1\t2\t3"
tva expr -n "a,b,c" -r "1,2,3" -E 'len(@0)'       # Returns: 5 (length of "1\t2\t3")

# Column name with spaces
tva expr -n "user name" -r "John Doe" -E '@"user name"'  # Returns: John Doe
```

## Variable Binding

Use `as` keyword to bind expression results to variables. The `as` form returns the value of the expression,
allowing it to be used in subsequent operations or piped to functions.

```bash
# Basic syntax: bind calculation result
tva expr -n "price,qty,tax_rate" -r "10,5,0.1" -E '@price * @qty as @total; @total * (1 + @tax_rate)'
# Returns: 55

# Reuse intermediate results
tva expr -n "name" -r "John Smith" -E '@name | split(" ") as @parts; first(@parts) ++ "." ++ last(@parts)'
# Returns: John.Smith

# Multiple variable bindings
tva expr -n "price,qty" -r "10,5" -E '@price as @p; @qty as @q; @p * @q'
# Returns: 50

# Binding with pipe operations
tva expr -E '[1, 2, 3] as @list | len()'          # Returns: 3
tva expr -E '[1, 2, 3] as @list | len()'          # Returns: 3

# Chain method calls after binding
tva expr -E '("hello" as @s).upper()'     # Returns: HELLO
```

### Variable Scope

- Variables are valid within the current row only
- Variables are cleared when processing the next row
- Variables can shadow column references
- Variables can be rebound (reassigned)

```bash
# Variable shadows column
tva expr -n "price" -r "100" -E '
    @price *2 as @price;     // Column @price (100) bound to variable @price
    @price             // Variable @price (now 200)
'
# Returns: 200

# Variable rebinding
tva expr -n "price" -r "10" -E '
    @price as @p;         # @p = 10
    @p * 2 as @p;         # @p = 20 (rebound)
    @p * 2 as @p;         # @p = 40 (rebound again)
    @p
'
# Returns: 40
```

### Resolution Order

When evaluating `@name`, the engine checks in this order:

1. **Lambda parameters** - If inside a lambda, check lambda parameters first
2. **Variables** - Check variables bound with `as`
3. **Column names** - Fall back to column name lookup

**Design notes**:

- Unified `@` prefix reduces cognitive burden
- References jq syntax but removes `$` to avoid Shell conflicts

```bash
# Resolution order example
tva expr -n "x" -r "100" -E '
    @x as @y;             # Variable @y = column @x (100)
    map([1, 2, 3], x => x + @y)  # Lambda param x shadows nothing; @y is variable
'
# Returns: [101, 102, 103]
```

## Lambda Parameters

Lambda expressions introduce their own parameter scope:

```bash
# Lambda parameter shadows outer scope
tva expr -E '
    10 as @x;
    map([1, 2, 3], x => x + @x)  # Lambda param x; @x is variable (10)
'
# Returns: [11, 12, 13]

# Lambda captures outer variables
tva expr -E '
    5 as @offset;
    map([1, 2, 3], n => n + @offset)  # @offset is captured from outer scope
'
# Returns: [6, 7, 8]
```

Lambda parameters:
- Do not use `@` prefix (distinguishes from columns/variables)
- Are lexically scoped
- Can capture variables from outer scope

## Expression Separator

`;` - Separates multiple expressions. Expressions are evaluated in order, and the value of
the last expression is returned.

```bash
# Multiple expressions: bind then use the variable
tva expr -E '[1, 2, 3] as @list; @list | len()'  # Returns: 3

# Calculate and reuse
tva expr -E '@price * @qty as @total; @total * 1.1' -n "price,qty" -r "100,2"
# Returns: 220 (100*2=200, then 200*1.1=220)
```

## Best Practices

1. **Use descriptive variable names**: `@total_price` instead of `@tp`
2. **Avoid unnecessary shadowing**: Can be confusing
3. **Bind early, use often**: Reduces repetition and improves readability
4. **Document complex pipelines**: Use comments with `//`

```bash
# Good: clear variable names
tva expr -n "price,qty,discount" -r "100,5,0.1" -E '
    @price * @qty as @subtotal;            // Calculate subtotal
    @subtotal * (1 - @discount) as @total; // Apply discount
    @total
'
# Returns: 450

# Avoid: unclear one-letter names
tva expr -n "price,qty,discount" -r "100,5,0.1" -E '@price * @qty as @a; @a * (1 - @discount)'
```
