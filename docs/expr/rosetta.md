# Rosetta Code with TVA Expression Engine

This document demonstrates the capabilities of TVA's expression engine by implementing tasks from [Rosetta Code](https://rosettacode.org/).

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

### 99 Bottles of Beer

Display the complete lyrics for the song: 99 Bottles of Beer on the Wall.

Using `range()` and string concatenation:

```bash
tva expr -E '
join(
    map(
        range(99, 0, -1),
        n => n ++ " bottles of beer on the wall,\n" ++
        n ++ " bottles of beer!\n" ++
        "Take one down, pass it around,\n" ++
        (n - 1) ++ " bottles of beer on the wall!"
    ),
    "\n\n"
)'
```

This demonstrates:
- `range(99, 0, -1)` - Generate countdown from 99 to 1
- `.map()` method with lambda - Transform each number to a verse
- `++` for string concatenation
- `.join()` method to combine verses with double newlines

### FizzBuzz

Write a program that prints the integers from 1 to 100 (inclusive). But for multiples of three, print "Fizz" instead of the number; for multiples of five, print "Buzz"; for multiples of both three and five, print "FizzBuzz".

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
) | join(_, "\n")'
```

This demonstrates:
- `range(1, 101)` - Generate numbers from 1 to 100
- Nested `if()` for multiple conditions
- Modulo operator `%` for divisibility checks
- `.join("\n")` to output one item per line

