# Expression Variables

## Column References

Use `@` prefix to reference columns, avoiding conflicts with Shell variables:

| Syntax | Description | Example |
| :--- | :--- | :--- |
| `@0` | Entire row content | `@0` represents all columns |
| `@1`, `@2` | 1-based column index | `@1` represents the first column |
| `@col_name` | Column name reference | `@price` represents the price column |
| ` @"col name"` | Column name with spaces | ` @"user name"` represents column "user name" |

**Design rationale**:
- **Shell-friendly**: `@` has no special meaning in bash/zsh, no escaping needed
- **Concise**: Only 2 characters (`Shift+2`)

**Type behavior**:
- Column references return `String` by default (raw bytes)
- Numeric operations automatically attempt parsing; failure yields `null`
- Use `int(@col)` or `float(@col)` for explicit type specification

## Variable Binding

Use `as` keyword to bind expression results to variables for reuse in subsequent pipes:

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
```

**Rules**:
- Variables are valid within the current row only, cleared when entering next row
- Can shadow column references (`@price as @price`)
- Column references check variables first, then fall back to column name lookup

**Design notes**:
- `as` aligns with pipe semantics ("name the result from the left...")
- Unified `@` prefix reduces cognitive burden
- References jq syntax but removes `$` to avoid Shell conflicts
