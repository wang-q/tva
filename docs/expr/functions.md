# Expr Functions

TVA expr engine provides a rich set of built-in functions for data processing.

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

```bash
# Basic numeric operations
tva expr -E 'abs(-42)'                      # Returns: 42
tva expr -E 'ceil(3.14)'                    # Returns: 4
tva expr -E 'floor(3.14)'                   # Returns: 3
tva expr -E 'round(3.5)'                    # Returns: 4
tva expr -E 'sqrt(16)'                      # Returns: 4

# Power and logarithm
tva expr -E 'pow(2, 10)'                    # Returns: 1024
tva expr -E 'ln(1)'                         # Returns: 0
tva expr -E 'log10(100)'                    # Returns: 2
tva expr -E 'exp(0)'                        # Returns: 1

# Min and max
tva expr -E 'max(1, 5, 3, 9, 2)'            # Returns: 9
tva expr -E 'min(1, 5, 3, -2, 2)'           # Returns: -2

# Type conversions
tva expr -E 'int("42")'                     # Returns: 42
tva expr -E 'float("3.14")'                 # Returns: 3.14

# Trigonometric functions
tva expr -E 'sin(0)'                        # Returns: 0
tva expr -E 'cos(0)'                        # Returns: 1
tva expr -E 'tan(0)'                        # Returns: 0
```

## String Manipulation

- trim(string) -> string: Remove leading and trailing whitespace
- upper(string) -> string: Convert to uppercase
- lower(string) -> string: Convert to lowercase
- char_len(string) -> int: String character count (UTF-8)
- substr(string, start, len) -> string: Substring
- split(string, pat) -> list: Split string by pattern
- contains(value, item) -> bool: Check if string contains substring, or list contains element
- starts_with(string, prefix) -> bool: Check if string starts with prefix
- ends_with(string, suffix) -> bool: Check if string ends with suffix
- replace(string, from, to) -> string: Replace substring
- truncate(string, len, end?) -> string: Truncate string
- wordcount(string) -> int: Word count
- fmt(template, ...args) -> string: Format string with placeholders

See [String Formatting (fmt)](fmt.md) for detailed documentation.

```bash
# String manipulation examples
tva expr -E 'trim("  hello  ")'             # Returns: "hello"
tva expr -E 'upper("hello")'                # Returns: "HELLO"
tva expr -E 'lower("WORLD")'                # Returns: "world"
tva expr -E 'len("hello")'                  # Returns: 5
tva expr -E 'char_len("你好")'               # Returns: 2 (UTF-8 characters)
tva expr -E 'substr("hello world", 0, 5)'   # Returns: "hello"

tva expr -E 'split("1,2,3", ",")'           # Returns: ["1", "2", "3"]
tva expr -E 'split("1,2,3", ",") | join(_, "-")'  # Returns: "1-2-3"

tva expr -E 'contains("hello", "ll")'       # Returns: true
tva expr -E 'starts_with("hello", "he")'    # Returns: true
tva expr -E 'ends_with("hello", "lo")'      # Returns: true

tva expr -E 'replace("hello", "l", "x")'    # Returns: "hexxo"
tva expr -E 'truncate("hello world", 5)'    # Returns: "he..."
tva expr -E 'wordcount("hello world")'      # Returns: 2

# fmt() - String formatting (see fmt.md for complete documentation)
tva expr -E 'fmt("Hello %()!", "World")'                    # Returns: "Hello World!"
tva expr -E 'fmt("%(1) has %(2) points", "Alice", 100)'      # Returns: "Alice has 100 points"
tva expr -E 'fmt("Hex: %(1:#x)", 255)'                       # Returns: "Hex: 0xff"

# Column references with %(@n)
tva expr -E 'fmt("%(@1) has %(@2) points")' -r "Alice,100"

# Lambda variable references
tva expr -E 'map([1, 2, 3], x => fmt("value: %(x)"))'

# Using different delimiters to avoid conflicts
tva expr -E 'fmt(q(The "value" is %[1]), 42)'
```

## Generic Functions

These functions have different implementations for different argument types.
The implementation is selected at runtime based on the first argument type.

- len(value) -> int: Returns length of string (bytes) or list (element count)
- is_empty(value) -> bool: Check if string or list is empty
- contains(value, item) -> bool: Check if string contains substring, or list contains element
- take(value, n) -> T: Take first n elements from string or list
- drop(value, n) -> T: Drop first n elements from string or list
- concat(value1, value2, ...) -> T: Concatenate strings or lists

```bash
# Check if string/list is empty
tva expr -E 'is_empty("")'                # Returns: true
tva expr -E 'is_empty("hello")'           # Returns: false
tva expr -E 'is_empty([])'                # Returns: true
tva expr -E 'is_empty([1, 2, 3])'         # Returns: false

# Take first n elements from string or list
tva expr -E 'take("hello", 3)'            # Returns: "hel"
tva expr -E 'take([1, 2, 3, 4, 5], 3)'    # Returns: [1, 2, 3]

# Drop first n elements from string or list
tva expr -E 'drop("hello", 2)'            # Returns: "llo"
tva expr -E 'drop([1, 2, 3, 4, 5], 2)'    # Returns: [3, 4, 5]

# Concatenate multiple strings or lists
tva expr -E 'concat("hello", " ", "world")'  # Returns: "hello world"
tva expr -E 'concat([1, 2], [3, 4], [5, 6])'   # Returns: [1, 2, 3, 4, 5, 6]
```

## Range Generation

- range(upto) -> list: Generate numbers from 0 to upto (exclusive), step 1
- range(from, upto) -> list: Generate numbers from from (inclusive) to upto (exclusive), step 1
- range(from, upto, by) -> list: Generate numbers from from (inclusive) to upto (exclusive), step by

The range function produces a list of numbers. Similar to jq's range:

```bash
tva expr -E 'range(4) | join(_, ", ")'          # Returns: "0, 1, 2, 3"
tva expr -E 'range(2, 5) | join(_, ", ")'        # Returns: "2, 3, 4"
tva expr -E 'range(0, 10, 3) | join(_, ", ")'    # Returns: "0, 3, 6, 9"
tva expr -E 'range(0, -5, -1) | join(_, ", ")'   # Returns: "0, -1, -2, -3, -4"
```

Note: If step direction doesn't match the range direction (e.g., positive step with from > upto),
returns empty list.

## List Operations

- first(list) -> T: First element
- join(list, sep) -> string: Join list elements
- last(list) -> T: Last element
- nth(list, n) -> T: nth element (0-based, negative indices return null)
- reverse(list) -> list: Reverse list
- replace_nth(list, n, value) -> list: Return new list with nth element replaced by value (original list unchanged)
- slice(list, start, end?) -> list: Slice list
- sort(list) -> list: Sort list
- unique(list) -> list: Remove duplicates
- flatten(list) -> list: Flatten nested list by one level
- zip(list1, list2, ...) -> list: Zip multiple lists into list of tuples
- grouped(list, n) -> list: Group list into chunks of size n

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

# Flatten nested list
tva expr -E 'flatten([[1, 2], [3, 4]])'        # Returns: [1, 2, 3, 4]
tva expr -E 'flatten([[1, 2], 3, [4, 5]])'     # Returns: [1, 2, 3, 4, 5]

# Zip multiple lists
tva expr -E 'zip([1, 2], ["a", "b"])'          # Returns: [[1, "a"], [2, "b"]]
tva expr -E 'zip([1, 2, 3], ["a", "b"])'       # Returns: [[1, "a"], [2, "b"]] (truncated to shortest)

# Partition list by predicate
tva expr -E 'partition([1, 2, 3, 4], x -> x % 2 == 0)'   # Returns: [[2, 4], [1, 3]]
tva expr -E 'partition([1, 2, 3, 4, 5], x -> x > 3)'     # Returns: [[4, 5], [1, 2, 3]]

# Flat map (map then flatten)
tva expr -E 'flat_map([1, 2], x -> [x, x * 2])'          # Returns: [1, 2, 2, 4]
tva expr -E 'flat_map(["a", "b"], x -> split(x, ""))'    # Returns: ["a", "b"]

# Group list into chunks
tva expr -E 'grouped([1, 2, 3, 4, 5], 2)'      # Returns: [[1, 2], [3, 4], [5]]
tva expr -E 'grouped([1, 2, 3, 4], 2)'         # Returns: [[1, 2], [3, 4]]
```

## Logic & Control

- if(cond, then, else?) -> T: Conditional expression, returns then if cond is true, else otherwise (
  or null)
- default(val, fallback) -> T: Returns fallback if val is null or empty

```bash
# Conditional expressions
tva expr -E 'if(true, "yes", "no")'       # Returns: "yes"
tva expr -E 'if(false, "yes", "no")'      # Returns: "no"

# Default values for null/empty
tva expr -E 'default(null, "fallback")'     # Returns: "fallback"
```

## Higher-Order Functions

- map(list, lambda) -> list: Apply lambda to each element
- filter(list, lambda) -> list: Filter list elements
- filter_index(list, lambda) -> list: Return indices of elements satisfying the predicate
- reduce(list, init, lambda) -> value: Reduce list to single value
- sort_by(list, lambda) -> list: Sort list by lambda expression
- take_while(list, lambda) -> list: Take elements while lambda is true
- partition(list, lambda) -> list: Partition list into [satisfying, not_satisfying]
- flat_map(list, lambda) -> list: Map and flatten result by one level

```bash
# Double each number
tva expr -E 'map([1, 2, 3], x => x * 2) | join(_, ", ")'
# Returns: "2, 4, 6"

# Keep numbers greater than 2
tva expr -E 'filter([1, 2, 3, 4], x => x > 2) | join(_, ", ")'
# Returns: "3, 4"

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

# Take elements while condition is true
tva expr -E 'take_while([1, 2, 3, 4, 5], x => x < 4)'
# Returns: [1, 2, 3]

# Take elements from start while they are even
tva expr -E 'take_while([2, 4, 6, 7, 8, 10], x => x % 2 == 0)'
# Returns: [2, 4, 6]

# Take strings while they start with "a"
tva expr -E 'take_while(["apple", "apricot", "banana", "avocado"], s => s.starts_with("a"))'
# Returns: ["apple", "apricot"]

# Find indices of elements satisfying condition
tva expr -E 'filter_index([10, 15, 20, 25, 30], x => x > 18)'
# Returns: [2, 3, 4]

# Find indices of even numbers
tva expr -E 'filter_index([1, 2, 3, 4, 5], x => x % 2 == 0)'
# Returns: [1, 3]

# Concatenate lists
tva expr -E 'concat([1, 2], [3, 4]) | join(_, ", ")'
# Returns: "1, 2, 3, 4"

# Concatenate strings (alternative to ++ operator)
tva expr -E 'concat("hello", " ", "world")'
# Returns: "hello world"
```

## Regular Expressions

*Note: Regex operations can be expensive, use with caution.*

- regex_match(string, pattern) -> bool: Check if matches regex
- regex_extract(string, pattern, group?) -> string: Extract capture group
- regex_replace(string, pattern, to) -> string: Regex replace

```bash
# Check if string matches regex pattern
tva expr -E 'regex_match("hello", "h.*o")'           # Returns: true

# Extract capture group from string
tva expr -E 'regex_extract("hello world", "(\\w+)", 1)'  # Returns: "hello"

# Replace using regex
tva expr -E 'regex_replace("hello 123", "\\d+", "XXX")'  # Returns: "hello XXX"
```

## Encoding & Hashing

- md5(string) -> string: MD5 hash (hex)
- sha256(string) -> string: SHA256 hash (hex)
- base64(string) -> string: Base64 encode
- unbase64(string) -> string: Base64 decode

```bash
# MD5 hash
tva expr -E 'md5("hello")'           # Returns: "5d41402abc4b2a76b9719d911017c592"

# SHA256 hash
tva expr -E 'sha256("hello")'        # Returns: "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"

# Base64 encoding and decoding
tva expr -E 'base64("hello")'        # Returns: "aGVsbG8="
tva expr -E 'unbase64("aGVsbG8=")'   # Returns: "hello"
```

## Date & Time

- now() -> datetime: Current time
- strptime(string, format) -> datetime: Parse datetime
- strftime(datetime, format) -> string: Format datetime

```bash
# Current datetime
tva expr -E 'now()'                  # Returns: current datetime (e.g., "2026-03-19T10:30:00+08:00")

# Parse datetime from string (requires full datetime format)
tva expr -E 'strptime("2024-03-15T00:00:00", "%Y-%m-%dT%H:%M:%S")'           # Returns: datetime(2024-03-15T00:00:00)
tva expr -E 'strptime("15/03/2024 14:30:00", "%d/%m/%Y %H:%M:%S")'  # Returns: datetime(2024-03-15T14:30:00)

# Format datetime to string
tva expr -E 'strftime(now(), "%Y-%m-%d")'                   # Returns: "2026-03-19"
tva expr -E 'strftime(now(), "%H:%M:%S")'                   # Returns: "14:30:00"
tva expr -E 'strftime(strptime("2024-12-25T00:00:00", "%Y-%m-%dT%H:%M:%S"), "%B %d, %Y")'  # Returns: "December 25, 2024"

# Parse and format combined
tva expr -E 'strptime("2024-03-15T00:00:00", "%Y-%m-%dT%H:%M:%S") | strftime(_, "%d/%m/%Y")'  # Returns: "15/03/2024"
```

## IO

- print(val, ...): Print to stdout, returns last argument
- eprint(val, ...): Print to stderr, returns last argument

```bash
# Print to stdout (returns the value, so it can be used in expressions)
tva expr -E 'print("Hello", "World")'     # Prints: Hello World to stdout, returns: "World"
tva expr -E 'print(42)'                     # Prints: 42 to stdout, returns: 42
tva expr -E 'print("Result:", 1 + 2)'       # Prints: Result: 3 to stdout, returns: 3

# Print to stderr (useful for debugging)
tva expr -E 'eprint("Error message")'       # Prints: Error message to stderr, returns: "Error message"
tva expr -E 'eprint("Debug:", [1, 2, 3])'   # Prints: Debug: [1, 2, 3] to stderr

# Using print in pipelines
tva expr -E '[1, 2, 3] | print("List:", _) | len(_)'  # Prints: List: [1, 2, 3], returns: 3
```

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
