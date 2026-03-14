# Expression Functions & Operators

## Operators

### Operator Precedence (high to low)

1. `()` - Grouping
2. `-` (unary) - Negation
3. `**` - Power
4. `*`, `/`, `%` - Multiply, Divide, Modulo
5. `+`, `-` (binary) - Add, Subtract
6. `++` - String concatenation
7. `==`, `!=`, `<`, `<=`, `>`, `>=` - Numeric comparison
8. `eq`, `ne`, `lt`, `le`, `gt`, `ge` - String comparison
9. `not` - Logical NOT
10. `and` - Logical AND
11. `or` - Logical OR
12. `|` - Pipe

### Arithmetic Operators

- `-x`: Negation
- `a + b`: Addition
- `a - b`: Subtraction
- `a * b`: Multiplication
- `a / b`: Division
- `a % b`: Modulo
- `a ** b`: Power

### String Operators

- `a ++ b`: String concatenation

### Comparison Operators

**Numeric:**
- `a == b`: Equal
- `a != b`: Not equal
- `a < b`: Less than
- `a <= b`: Less than or equal
- `a > b`: Greater than
- `a >= b`: Greater than or equal

**String:**
- `a eq b`: String equal
- `a ne b`: String not equal
- `a lt b`: String less than (lexicographic)
- `a le b`: String less than or equal
- `a gt b`: String greater than
- `a ge b`: String greater than or equal

**Note**: No implicit type conversion. Use string comparison operators for string comparison.

### Logical Operators

- `not a`: Logical NOT
- `a and b`: Logical AND
- `a or b`: Logical OR

### Pipe Operator

- `a | f()`: Pipe left value to function (left value becomes first argument)
- `a | f(_, arg)`: Pipe with placeholder

---

## Functions

### Logic & Control

- if(cond, then, else?) -> T: Conditional expression, returns then if cond is true, else otherwise (or null)
- default(val, fallback) -> T: Returns fallback if val is null or empty

### Numeric Operations

- abs(x) -> number: Absolute value
- round(x) -> int: Round to nearest integer
- ceil(x) -> int: Ceiling (round up)
- floor(x) -> int: Floor (round down)
- sqrt(x) -> float: Square root
- pow(base, exp) -> float: Power operation
- min(a, b, *n) -> number: Minimum value
- max(a, b, *n) -> number: Maximum value
- int(val) -> int: Convert to integer, returns null on failure
- float(val) -> float: Convert to float, returns null on failure
- sin(x) -> float: Sine (radians)
- cos(x) -> float: Cosine (radians)
- tan(x) -> float: Tangent (radians)
- ln(x) -> float: Natural logarithm
- log10(x) -> float: Common logarithm (base 10)
- exp(x) -> float: Exponential function e^x

### String Manipulation

- trim(string) -> string: Remove leading and trailing whitespace
- upper(string) -> string: Convert to uppercase
- lower(string) -> string: Convert to lowercase
- len(string) -> int: String byte length
- char_len(string) -> int: String character count (UTF-8)
- substr(string, start, len) -> string: Substring
- split(string, pat) -> list: Split string by pattern
- contains(string, substr) -> bool: Check if string contains substring
- starts_with(string, prefix) -> bool: Check if string starts with prefix
- ends_with(string, suffix) -> bool: Check if string ends with suffix
- replace(string, from, to) -> string: Replace substring
- truncate(string, len, end?) -> string: Truncate string
- wordcount(string) -> int: Word count

### List Operations

*Note: These functions operate on expression `List` type (e.g., returned by `split()`), different from column-level aggregation in `stats` command.*

- first(list) -> T: First element
- last(list) -> T: Last element
- nth(list, n) -> T: nth element (0-based)
- join(list, sep) -> string: Join list elements
- reverse(list) -> list: Reverse list
- sort(list) -> list: Sort list
- unique(list) -> list: Remove duplicates
- slice(list, start, end?) -> list: Slice list

```bash
# Basic list operations
tva expr -E 'first([1, 2, 3])'           # Returns: 1
tva expr -E 'last([1, 2, 3])'            # Returns: 3
tva expr -E 'nth([1, 2, 3], 1)'          # Returns: 2 (0-based index)

# Using variables with multiple expressions
tva expr -E '
    [1, 2, 3] as @list;
    first(@list) + last(@list)
'
# Returns: 4
```

### Range Generation

- range(upto) -> list: Generate numbers from 0 to upto (exclusive), step 1
- range(from, upto) -> list: Generate numbers from from (inclusive) to upto (exclusive), step 1
- range(from, upto, by) -> list: Generate numbers from from (inclusive) to upto (exclusive), step by

The range function produces a list of numbers. Similar to jq's range:
- `range(4)` produces `[0, 1, 2, 3]`
- `range(2, 4)` produces `[2, 3]`
- `range(0, 10, 3)` produces `[0, 3, 6, 9]`
- `range(0, -5, -1)` produces `[0, -1, -2, -3, -4]`

Note: If step direction doesn't match the range direction (e.g., positive step with from > upto), returns empty list.

### Higher-Order Functions

- map(list, lambda) -> list: Apply lambda to each element
- filter(list, lambda) -> list: Filter list elements
- reduce(list, init, lambda) -> value: Reduce list to single value

Examples:

    map([1, 2, 3], |x| x * 2)        produces [2, 4, 6]
    filter([1, 2, 3, 4], |x| x > 2)  produces [3, 4]
    reduce([1, 2, 3], 0, |acc, x| acc + x)  produces 6

### Regular Expressions

*Note: Regex operations can be expensive, use with caution.*

- regex_match(string, pattern) -> bool: Check if matches regex
- regex_extract(string, pattern, group?) -> string: Extract capture group
- regex_replace(string, pattern, to) -> string: Regex replace

### Encoding & Hashing

- md5(string) -> string: MD5 hash (hex)
- sha256(string) -> string: SHA256 hash (hex)
- base64(string) -> string: Base64 encode
- unbase64(string) -> string: Base64 decode

### Date & Time

- now() -> datetime: Current time
- strptime(string, format) -> datetime: Parse datetime
- strftime(datetime, format) -> string: Format datetime

### IO

- print(val, ...): Print to stdout, returns last argument
- eprint(val, ...): Print to stderr, returns last argument
