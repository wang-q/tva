# Expression Functions

TVA expression engine provides a rich set of built-in functions for data processing.

## Numeric Operations

- abs(x) -> number: Absolute value
- ceil(x) -> int: Ceiling (round up)
- cos(x) -> float: Cosine (radians)
- exp(x) -> float: Exponential function e^x
- float(val) -> float: Convert to float, returns null on failure
- floor(x) -> int: Floor (round down)
- int(val) -> int: Convert to integer, returns null on failure
- ln(x) -> float: Natural logarithm
- log10(x) -> float: Common logarithm (base 10)
- max(a, b, *n) -> number: Maximum value
- min(a, b, *n) -> number: Minimum value
- pow(base, exp) -> float: Power operation
- round(x) -> int: Round to nearest integer
- sin(x) -> float: Sine (radians)
- sqrt(x) -> float: Square root
- tan(x) -> float: Tangent (radians)

## String Manipulation

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

```bash
# String manipulation examples
tva expr -E 'upper("hello")'                # Returns: "HELLO"
tva expr -E 'lower("WORLD")'                # Returns: "world"
tva expr -E 'len("hello")'                  # Returns: 5
tva expr -E 'char_len("你好")'               # Returns: 2 (UTF-8 characters)
tva expr -E 'substr("hello", 1, 3)'         # Returns: "ell"

tva expr -E 'split("1,2,3", ",")'           # Returns: ["1", "2", "3"]
tva expr -E 'split("1-2-3", "-").reverse()' # Returns: ["3", "2", "1"]

tva expr -E 'contains("hello", "ll")'       # Returns: true
tva expr -E 'starts_with("hello", "he")'    # Returns: true
tva expr -E 'ends_with("hello", "lo")'      # Returns: true

tva expr -E 'replace("hello", "l", "x")'    # Returns: "hexxo"
tva expr -E 'truncate("hello world", 5)'    # Returns: "he..."
tva expr -E 'wordcount("hello world")'      # Returns: 2
tva expr -E 'wordcount("one two three four")'  # Returns: 4
```

## List Operations

- first(list) -> T: First element
- join(list, sep) -> string: Join list elements
- last(list) -> T: Last element
- len(list) -> int: List length (number of elements)
- nth(list, n) -> T: nth element (0-based, negative indices return null)
- reverse(list) -> list: Reverse list
- replace_nth(list, n, value) -> list: Return new list with nth element replaced by value (original list unchanged)
- slice(list, start, end?) -> list: Slice list
- sort(list) -> list: Sort list
- unique(list) -> list: Remove duplicates

*Note: These functions operate on expression `List` type (e.g., returned by `split()`), different
from column-level aggregation in `stats` command.*

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

# List length
tva expr -E 'len([1, 2, 3, 4, 5])'        # Returns: 5
tva expr -E 'len(split("a,b,c", ","))'    # Returns: 3
tva expr -E '
    [1, 2, 3] as @list;
    @list.len()
'
# Returns: 3

# Replace element at index (returns new list, original unchanged)
tva expr -E 'replace_nth([1, 2, 3], 1, 99)'    # Returns: [1, 99, 3]
tva expr -E '
    [1, 2, 3] as @list;
    replace_nth(@list, 0, 100) as @new_list;
    [@list, @new_list]
'
# Returns: [[1, 2, 3], [100, 2, 3]]
```

## Range Generation

- range(upto) -> list: Generate numbers from 0 to upto (exclusive), step 1
- range(from, upto) -> list: Generate numbers from from (inclusive) to upto (exclusive), step 1
- range(from, upto, by) -> list: Generate numbers from from (inclusive) to upto (exclusive), step by

The range function produces a list of numbers. Similar to jq's range:

- `range(4)` produces `[0, 1, 2, 3]`
- `range(2, 4)` produces `[2, 3]`
- `range(0, 10, 3)` produces `[0, 3, 6, 9]`
- `range(0, -5, -1)` produces `[0, -1, -2, -3, -4]`

Note: If step direction doesn't match the range direction (e.g., positive step with from > upto),
returns empty list.

## Logic & Control

- if(cond, then, else?) -> T: Conditional expression, returns then if cond is true, else otherwise (
  or null)
- default(val, fallback) -> T: Returns fallback if val is null or empty

## Higher-Order Functions

- map(list, lambda) -> list: Apply lambda to each element
- filter(list, lambda) -> list: Filter list elements
- reduce(list, init, lambda) -> value: Reduce list to single value
- sort_by(list, lambda) -> list: Sort list by lambda expression

```bash
# Double each number
tva expr -E 'map([1, 2, 3], x => x * 2)'
# Returns: [2, 4, 6]

# Keep numbers greater than 2
tva expr -E 'filter([1, 2, 3, 4], x => x > 2)'
# Returns: [3, 4]

# Sum all numbers (0 + 1 + 2 + 3)
tva expr -E 'reduce([1, 2, 3], 0, (acc, x) => acc + x)'
# Returns: 6

# Count elements in a list
tva expr -E 'reduce(["a", "b", "c"], 0, (acc, _) => acc + 1)'
# Returns: 3

# Find maximum value
tva expr -E 'reduce([3, 1, 4, 1, 5], 0, (acc, x) => if(x > acc, x, acc))'
# Returns: 5

# Sort by string length
tva expr -E 'sort_by(["cherry", "apple", "pear"], s => len(s))'
# Returns: ["pear", "apple", "cherry"]

# Sort by absolute value
tva expr -E 'sort_by([-5, 3, -1, 4], x => abs(x))'
# Returns: [-1, 3, 4, -5]

# Sort records by first element
tva expr -E 'sort_by([[3, "c"], [1, "a"], [2, "b"]], r => r.first())'
# Returns: [[1, "a"], [2, "b"], [3, "c"]]

# Sort strings case-insensitively
tva expr -E 'sort_by(["Banana", "apple", "Cherry"], s => lower(s))'
# Returns: ["apple", "Banana", "Cherry"]

# Sort by multiple criteria (composite key)
tva expr -E 'sort_by([[2, "b"], [1, "c"], [1, "a"]], r => [r.nth(0), r.nth(1)])'
# Returns: [[1, "a"], [1, "c"], [2, "b"]]
```

## Regular Expressions

*Note: Regex operations can be expensive, use with caution.*

- regex_match(string, pattern) -> bool: Check if matches regex
- regex_extract(string, pattern, group?) -> string: Extract capture group
- regex_replace(string, pattern, to) -> string: Regex replace

## Encoding & Hashing

- md5(string) -> string: MD5 hash (hex)
- sha256(string) -> string: SHA256 hash (hex)
- base64(string) -> string: Base64 encode
- unbase64(string) -> string: Base64 decode

## Date & Time

- now() -> datetime: Current time
- strptime(string, format) -> datetime: Parse datetime
- strftime(datetime, format) -> string: Format datetime

## IO

- print(val, ...): Print to stdout, returns last argument
- eprint(val, ...): Print to stderr, returns last argument

## Meta Functions

- type(value) -> string: Returns the type name of the value
  - Returns: "int", "float", "string", "bool", "null", or "list"
- is_null(value) -> bool: Returns true if value is null
- is_int(value) -> bool: Returns true if value is an integer
- is_float(value) -> bool: Returns true if value is a float
- is_numeric(value) -> bool: Returns true if value is int or float
- is_string(value) -> bool: Returns true if value is a string
- is_bool(value) -> bool: Returns true if value is a boolean
- is_list(value) -> bool: Returns true if value is a list

- env(name) -> string: Get environment variable value
  - Returns `null` if variable not set
- cwd() -> string: Returns the current working directory
- version() -> string: Returns the TVA version
- platform() -> string: Returns the operating system name
  - Returns: "windows", "macos", "linux", or "unknown"

```bash
# type() examples
tva expr -E '[[1,2], "string", true, null, -5]'
# [List([Int(1), Int(2)]), String("string"), Bool(true), Null, Int(-5)]

tva expr -E '[[1,2], "string", true, null, -5, x => x + 1].map(x => type(x)).join(",")'
# list,string,bool,null,int,lambda

# Type checking functions
tva expr -E 'is_null(null)'                # Returns: true
tva expr -E 'is_null("hello")'             # Returns: false
tva expr -E 'is_int(42)'                   # Returns: true
tva expr -E 'is_int(3.14)'                 # Returns: false
tva expr -E 'is_float(3.14)'               # Returns: true
tva expr -E 'is_numeric(42)'               # Returns: true
tva expr -E 'is_numeric(3.14)'             # Returns: true
tva expr -E 'is_string("hello")'           # Returns: true
tva expr -E 'is_bool(true)'                # Returns: true
tva expr -E 'is_list([1, 2, 3])'           # Returns: true

# env() examples
tva expr -E 'env("HOME")'        # Returns: "/home/user"
tva expr -E 'env("PATH")'        # Returns: "/usr/bin:/bin"
tva expr -E 'default(env("DEBUG"), "false")'  # Returns: "false" (if DEBUG not set)

# version() and platform() examples
tva expr -E 'version()'          # Returns: "0.2.5"
tva expr -E 'platform()'         # Returns: "windows" / "macos" / "linux"

# cwd() example
tva expr -E 'cwd()'              # Returns: "/path/to/current/dir"

```
