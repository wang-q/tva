# Rosetta Code Examples

This document demonstrates the capabilities of TVA's expression engine by implementing tasks
from [Rosetta Code](https://rosettacode.org/).

## Tasks

### Hello World

Display the string "Hello world!" on a text console.

```bash
tva expr -E '"Hello world!"'
```

Output:

```
Hello world!
```

This demonstrates:

- `tva expr` - Command for standalone expression evaluation
- The result of the last expression is printed to stdout

### 99 Bottles of Beer

Display the complete lyrics for the song: 99 Bottles of Beer on the Wall.

Using `range()` and string concatenation:

```bash
tva expr -E '
map(
    range(99, 0, -1),
    n => 
    n ++ " bottles of beer on the wall,\n" ++
    n ++ " bottles of beer!\n" ++
    "Take one down, pass it around,\n" ++
    (n - 1) ++ " bottles of beer on the wall!\n"
) | join(_, "\n")
'
```

This demonstrates:

- `range(99, 0, -1)` - Generate countdown from 99 to 1
- `.map()` method with lambda - Transform each number to a verse
- `++` for string concatenation
- `.join()` method to combine verses with double newlines

### FizzBuzz

Write a program that prints the integers from 1 to 100 (inclusive). But for multiples of three,
print "Fizz" instead of the number; for multiples of five, print "Buzz"; for multiples of both three
and five, print "FizzBuzz".

```bash
tva expr -E '
map(
    range(1, 101),
    n =>
    if(n % 15 == 0, "FizzBuzz",
        if(n % 3 == 0, "Fizz",
            if(n % 5 == 0, "Buzz", n)
        )
    )
) | join(_, "\n")
'
```

This demonstrates:

- `range(1, 101)` - Generate numbers from 1 to 100
- Nested `if()` for multiple conditions
- Modulo operator `%` for divisibility checks
- `.join("\n")` to output one item per line

### Factorial

The factorial of 0 is defined as 1. The factorial of a positive integer n is defined as the product
n × (n-1) × (n-2) × ... × 1.

Using `reduce()` for iterative approach:

```bash
# Factorial of 5: 5! = 5 × 4 × 3 × 2 × 1 = 120
tva expr -E 'reduce(range(1, 6), 1, (acc, n) => acc * n)'
```

Output:

```
120
```

Computing factorials for 0 through 10:

```bash
tva expr -E 'map(
    range(0, 11),
    n => if(n == 0, 1, 
        reduce(range(1, n + 1), 1, (acc, x) => acc * x))
) | join(_, "\n")'
```

This demonstrates:

- `reduce(list, init, op)` - Aggregate list values with an accumulator
- Lambda with two parameters `(acc, n)` for accumulator and current item
- Special case handling for `0! = 1`

### Fibonacci sequence

The Fibonacci sequence is a sequence Fn of natural numbers defined recursively:

- F0 = 0
- F1 = 1
- Fn = Fn-1 + Fn-2, if n > 1

Generate the first 20 Fibonacci numbers:

```bash
tva expr -E '
map(
    range(0, 20),
    n => if(n == 0, 0,
        if(n == 1, 1,
            reduce(
                range(2, n + 1),
                [0, 1],
                (acc, _) => [acc.nth(1), acc.nth(0) + acc.nth(1)]
            ).nth(1)
        )
    )
) | join(_, ", ")
'
```

Output:

```
0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987, 1597, 2584, 4181
```

This demonstrates:

- Iterative Fibonacci computation using `reduce()`
- Tuple-like list `[prev, curr]` to track state
- List access with `.nth()` method to get previous values
- `range(2, n + 1)` to iterate (n-1) times for the nth Fibonacci number

### Palindrome detection

A palindrome is a phrase which reads the same backward and forward.

Check if a string is a palindrome:

```bash
tva expr -E '
"A man, a plan, a canal: Panama" |
    lower() |
    regex_replace(_, "[^a-z0-9]", "") as @cleaned;
@cleaned.split("").reverse().join("") as @reversed;
@cleaned == @reversed
'
```

Output:

```
true
```

This demonstrates:

- `lower()` - Convert to lowercase for case-insensitive comparison
- `regex_replace()` - Remove non-alphanumeric characters
- `as @var` - Bind intermediate results to variables
- Method chaining - `split().reverse().join()` to reverse a string

### Word frequency

Given a text file and an integer `n`, print/display the `n` most common words in the file (and the
number of their occurrences) in decreasing frequency.

```bash
tva expr -E '
"the quick brown fox jumps over the lazy dog the quick brown fox" |
    lower() |
    split(_, " ") as @words;

// Get unique words
@words | unique() as @unique_words;

// Count occurrences of each unique word
// Note: Lambda body must be a single expression, so we use nested function calls
map(@unique_words, word =>
    [word, filter(@words, w => w == word) | len()]
) as @word_counts;

// Sort by count in descending order
sort_by(@word_counts, pair => [-pair.nth(1), pair.nth(0)])
    .map(pair => pair.join(": "))
    .join("\n")
'
```

Output:

```
the: 3
brown: 2
fox: 2
quick: 2
dog: 1
jumps: 1
lazy: 1
over: 1
```

This demonstrates:

- `unique()` - Remove duplicate words
- Nested `map` and `filter` - For each unique word, count occurrences
- `len()` - Get list length as count
- List construction - Build `[word, count]` pairs
- `sort_by()` - Sort by frequency (using negation for descending order)

### Sieve of Eratosthenes

Implement the Sieve of Eratosthenes algorithm, with the only allowed optimization that the outer loop can stop at the square root of the limit, and the inner loop may start at the square of the prime just found.

Find all prime numbers up to 100:

```bash
tva expr  -r '100' -E '
int(@1) as @limit;
int(sqrt(@limit)) as @sqrt_limit;

// Initialize: all numbers >= 2 are potentially prime
map(range(0, @limit + 1), n => n >= 2) as @is_prime;

// Sieve: for each prime p, mark its multiples as not prime
// Outer loop stops at sqrt(limit), inner loop starts at p*p
reduce(
    range(2, @sqrt_limit + 1),
    @is_prime,
    (primes, p) =>
        if(primes.nth(p),
            reduce(
                range(p * p, @limit + 1, p),
                primes,
                (acc, m) => acc.replace_nth(m, false)
            ),
            primes
        )
) as @sieved;

// Collect all prime numbers
filter(range(2, @limit + 1), n => @sieved.nth(n)) |
    join(_, ", ")
'
```

Output:

```
2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97
```

This demonstrates:

- `sqrt()` and `int()` - Calculate square root for outer loop limit
- Boolean list as sieve - Index represents number, value represents primality
- Nested `reduce()` - Outer loop iterates candidates, inner loop marks multiples
- `replace_nth()` - Immutable list update for marking composites
- `filter()` with predicate - Collect numbers where sieve value is true
- Optimization: inner loop starts at `p * p` (smaller multiples already marked)

### Greatest common divisor

Find the greatest common divisor (GCD) of two integers.

Using `take_while()` to find the GCD by searching from largest to smallest:

```bash
# GCD of 48 and 18: gcd(48, 18) = 6
tva expr -r '48,18' -E '
int(@1) as @a;
int(@2) as @b;
min(@a, @b) as @limit;

// Generate candidates from largest to smallest
reverse(range(1, @limit + 1)) as @candidates;

// Take while we haven not found a common divisor yet
// Then get the first one that is a common divisor
take_while(@candidates, d => @a % d != 0 or @b % d != 0) as @not_common;
len(@not_common) as @skip_count;
nth(@candidates, @skip_count)
'
```

Output:

```
6
```

This demonstrates:

- `take_while()` to skip non-divisors until finding the GCD
- `reverse()` to search from largest to smallest for efficiency
- `nth()` with calculated offset to extract the first matching element
