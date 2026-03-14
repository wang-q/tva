# Rosetta Code with TVA Expression Engine

This document demonstrates the capabilities of TVA's expression engine by implementing tasks
from [Rosetta Code](https://rosettacode.org/).

## Basic Usage

Most examples use the `tva expr` command for standalone expression evaluation:

```bash
# Simple arithmetic
tva expr -E "2 + 2"

# With input data
tva expr -n "x,y" -r "3,4" -E "@x * @y"

# Using higher-order functions
tva expr -E "map([1,2,3,4,5], x => x * x)"
```

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

Count unique words (simplified version without sorting):

```bash
tva expr -E '
"the quick brown fox jumps over the lazy dog the quick brown fox"
| lower()
| split(_, " ") as @words;

// Get unique words
@words | unique() as @unique_words;

// Count occurrences of each unique word
map(@unique_words, word =>
    filter(@words, w => w == word) | len() as @count |
    [word, @count]
)
'
```

Output:

```
[["the", 3], ["quick", 2], ["brown", 2], ["fox", 2], ["jumps", 1], ["over", 1], ["lazy", 1], ["dog", 1]]
```

This demonstrates:

- `unique()` - Remove duplicate words
- Nested `map` and `filter` - For each unique word, count occurrences
- `len()` - Get list length as count
- List construction - Build `[word, count]` pairs

## list

进阶级（字符串、数据结构、算法）
Word count
Prime numbers
Greatest common divisor
Sorting algorithms / Bubble sort / Quick sort
Binary search
Stack
Queue
Linked list / Reverse a linked list
高级 / 工程级（文件、并发、经典问题）
Read a file line by line
CSV manipulation
Concurrent computing / Print in order
Towers of Hanoi
Eight queens puzzle
Conway's Game of Life