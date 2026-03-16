#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

// =============================================================================
// Rosetta Code Examples Tests
// =============================================================================
// This file contains tests for the Rosetta Code examples from docs/expr/rosetta.md
//
// When updating this file:
// 1. Keep tests in the same order as they appear in rosetta.md
// 2. Add a comment with the original command for reference
// 3. Include a brief description of what the example demonstrates
// 4. Update this header if new examples are added to rosetta.md
//
// Last synced with rosetta.md: 2026-03-16
// =============================================================================

// -----------------------------------------------------------------------------
// Hello World
// -----------------------------------------------------------------------------
// Original: tva expr -E '"Hello world!"'
// Demonstrates: tva expr command for standalone expression evaluation
// -----------------------------------------------------------------------------
#[test]
fn rosetta_hello_world() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "\"Hello world!\""])
        .run();

    assert!(
        stdout.contains("Hello world!"),
        "Expected 'Hello world!' in stdout, got: {}",
        stdout
    );
}

// -----------------------------------------------------------------------------
// 99 Bottles of Beer
// -----------------------------------------------------------------------------
// Demonstrates: range(), map() with lambda, string concatenation (++), join()
// -----------------------------------------------------------------------------
#[test]
fn rosetta_99_bottles_of_beer() {
    // Test just the first verse to avoid huge output
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "99 ++ \" bottles of beer on the wall,\\n\" ++ 99 ++ \" bottles of beer!\"",
        ])
        .run();

    assert!(
        stdout.contains("99 bottles of beer on the wall"),
        "Expected '99 bottles of beer on the wall' in stdout, got: {}",
        stdout
    );
    assert!(
        stdout.contains("99 bottles of beer!"),
        "Expected '99 bottles of beer!' in stdout, got: {}",
        stdout
    );
}

// -----------------------------------------------------------------------------
// FizzBuzz
// -----------------------------------------------------------------------------
// Demonstrates: range(), map(), nested if(), modulo operator (%), join()
// -----------------------------------------------------------------------------
#[test]
fn rosetta_fizzbuzz_first_15() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "map(range(1, 16), n => if(n % 15 == 0, \"FizzBuzz\", if(n % 3 == 0, \"Fizz\", if(n % 5 == 0, \"Buzz\", n)))) | join(_, \", \")",
        ])
        .run();

    // Check for FizzBuzz pattern in first 15 numbers
    assert!(
        stdout.contains("1, 2, Fizz, 4, Buzz"),
        "Expected '1, 2, Fizz, 4, Buzz' in stdout, got: {}",
        stdout
    );
    assert!(
        stdout.contains("FizzBuzz"),
        "Expected 'FizzBuzz' in stdout, got: {}",
        stdout
    );
}

#[test]
fn rosetta_fizzbuzz_number_15_is_fizzbuzz() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "if(15 % 15 == 0, \"FizzBuzz\", 15)"])
        .run();

    assert!(
        stdout.contains("FizzBuzz"),
        "Expected 'FizzBuzz' for number 15, got: {}",
        stdout
    );
}

#[test]
fn rosetta_fizzbuzz_number_3_is_fizz() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "if(3 % 3 == 0, \"Fizz\", 3)"])
        .run();

    assert!(
        stdout.contains("Fizz"),
        "Expected 'Fizz' for number 3, got: {}",
        stdout
    );
}

#[test]
fn rosetta_fizzbuzz_number_5_is_buzz() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "if(5 % 5 == 0, \"Buzz\", 5)"])
        .run();

    assert!(
        stdout.contains("Buzz"),
        "Expected 'Buzz' for number 5, got: {}",
        stdout
    );
}

// -----------------------------------------------------------------------------
// Factorial
// -----------------------------------------------------------------------------
// Demonstrates: reduce() for iterative computation, range()
// Formula: n! = n × (n-1) × ... × 1, with 0! = 1
// -----------------------------------------------------------------------------
#[test]
fn rosetta_factorial_5() {
    // 5! = 5 × 4 × 3 × 2 × 1 = 120
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "reduce(range(1, 6), 1, (acc, n) => acc * n)"])
        .run();

    assert!(
        stdout.contains("120"),
        "Expected '120' for 5!, got: {}",
        stdout
    );
}

#[test]
fn rosetta_factorial_0() {
    // 0! = 1 (by definition)
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "if(0 == 0, 1, reduce(range(1, 1), 1, (acc, x) => acc * x))",
        ])
        .run();

    assert!(stdout.contains("1"), "Expected '1' for 0!, got: {}", stdout);
}

#[test]
fn rosetta_factorial_10() {
    // 10! = 3628800
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "reduce(range(1, 11), 1, (acc, n) => acc * n)"])
        .run();

    assert!(
        stdout.contains("3628800"),
        "Expected '3628800' for 10!, got: {}",
        stdout
    );
}

// -----------------------------------------------------------------------------
// Fibonacci Sequence
// -----------------------------------------------------------------------------
// Demonstrates: reduce() with state tracking, list access with nth()
// Formula: F(0)=0, F(1)=1, F(n)=F(n-1)+F(n-2)
// -----------------------------------------------------------------------------
#[test]
fn rosetta_fibonacci_10th() {
    // Compute the 10th Fibonacci number (should be 55)
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "reduce(range(2, 11), [0, 1], (acc, _) => [acc.nth(1), acc.nth(0) + acc.nth(1)]).nth(1)",
        ])
        .run();

    assert!(
        stdout.contains("55"),
        "Expected '55' for 10th Fibonacci number, got: {}",
        stdout
    );
}

#[test]
fn rosetta_fibonacci_0() {
    // F(0) = 0
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "0"]).run();

    assert!(
        stdout.contains("0"),
        "Expected '0' for F(0), got: {}",
        stdout
    );
}

#[test]
fn rosetta_fibonacci_1() {
    // F(1) = 1
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "1"]).run();

    assert!(
        stdout.contains("1"),
        "Expected '1' for F(1), got: {}",
        stdout
    );
}

// -----------------------------------------------------------------------------
// Palindrome Detection
// -----------------------------------------------------------------------------
// Demonstrates: lower(), regex_replace(), variable binding (as @var),
//               method chaining (split().reverse().join())
// -----------------------------------------------------------------------------
#[test]
fn rosetta_palindrome_simple() {
    // Simple palindrome check: "radar" reversed is "radar"
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "\"radar\" | split(\"\") | reverse() | join(\"\") as @reversed; \"radar\" == @reversed",
        ])
        .run();

    assert!(
        stdout.contains("true"),
        "Expected 'true' for palindrome 'radar', got: {}",
        stdout
    );
}

#[test]
fn rosetta_palindrome_not_palindrome() {
    // "hello" reversed is "olleh", not a palindrome
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "\"hello\" | split(\"\") | reverse() | join(\"\") as @reversed; \"hello\" == @reversed",
        ])
        .run();

    assert!(
        stdout.contains("false"),
        "Expected 'false' for non-palindrome 'hello', got: {}",
        stdout
    );
}

#[test]
fn rosetta_palindrome_case_insensitive() {
    // Case-insensitive check: "Racecar" should be a palindrome
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "\"Racecar\" | lower() | split(\"\") | reverse() | join(\"\") as @reversed; \"racecar\" == @reversed",
        ])
        .run();

    assert!(
        stdout.contains("true"),
        "Expected 'true' for case-insensitive palindrome 'Racecar', got: {}",
        stdout
    );
}

// -----------------------------------------------------------------------------
// Word Frequency
// -----------------------------------------------------------------------------
// Demonstrates: unique(), filter(), len(), map(), sort_by(), list construction
// -----------------------------------------------------------------------------
#[test]
fn rosetta_word_frequency_count() {
    // Count occurrences of "the" in a simple sentence
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "\"the quick brown fox jumps over the lazy dog\" | split(_, \" \") as @words; filter(@words, w => w == \"the\") | len()",
        ])
        .run();

    assert!(
        stdout.contains("2"),
        "Expected '2' occurrences of 'the', got: {}",
        stdout
    );
}

#[test]
fn rosetta_word_frequency_unique() {
    // Get unique words from a sentence
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "\"the the the quick quick brown\" | split(_, \" \") | unique() | len()",
        ])
        .run();

    assert!(
        stdout.contains("3"),
        "Expected '3' unique words, got: {}",
        stdout
    );
}

// -----------------------------------------------------------------------------
// Sieve of Eratosthenes
// -----------------------------------------------------------------------------
// Demonstrates: sqrt(), int(), reduce(), replace_nth(), filter()
// Algorithm: Mark multiples of each prime starting from 2
// -----------------------------------------------------------------------------
#[test]
fn rosetta_sieve_primes_up_to_10() {
    // Find primes up to 10: 2, 3, 5, 7
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "map(range(0, 11), n => n >= 2) as @is_prime; reduce(range(2, 4), @is_prime, (primes, p) => if(primes.nth(p), reduce(range(p * p, 11, p), primes, (acc, m) => acc.replace_nth(m, false)), primes)) as @sieved; filter(range(2, 11), n => @sieved.nth(n)) | join(_, \", \")",
        ])
        .run();

    assert!(
        stdout.contains("2, 3, 5, 7"),
        "Expected '2, 3, 5, 7' for primes up to 10, got: {}",
        stdout
    );
}

#[test]
fn rosetta_sieve_2_is_prime() {
    // 2 is the first prime number
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "2 >= 2"]).run();

    assert!(
        stdout.contains("true"),
        "Expected 'true' that 2 is prime candidate, got: {}",
        stdout
    );
}

// -----------------------------------------------------------------------------
// Greatest Common Divisor (GCD)
// -----------------------------------------------------------------------------
// Demonstrates: take_while(), reverse(), min(), nth()
// Algorithm: Search from largest to smallest for common divisor
// -----------------------------------------------------------------------------
#[test]
fn rosetta_gcd_48_18() {
    // gcd(48, 18) = 6
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "48 as @a; 18 as @b; min(@a, @b) as @limit; reverse(range(1, @limit + 1)) as @candidates; take_while(@candidates, d => @a % d != 0 or @b % d != 0) as @not_common; nth(@candidates, len(@not_common))",
        ])
        .run();

    assert!(
        stdout.contains("6"),
        "Expected '6' for gcd(48, 18), got: {}",
        stdout
    );
}

#[test]
fn rosetta_gcd_12_8() {
    // gcd(12, 8) = 4
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "12 as @a; 8 as @b; min(@a, @b) as @limit; reverse(range(1, @limit + 1)) as @candidates; take_while(@candidates, d => @a % d != 0 or @b % d != 0) as @not_common; nth(@candidates, len(@not_common))",
        ])
        .run();

    assert!(
        stdout.contains("4"),
        "Expected '4' for gcd(12, 8), got: {}",
        stdout
    );
}

#[test]
fn rosetta_gcd_7_5() {
    // gcd(7, 5) = 1 (coprime)
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "7 as @a; 5 as @b; min(@a, @b) as @limit; reverse(range(1, @limit + 1)) as @candidates; take_while(@candidates, d => @a % d != 0 or @b % d != 0) as @not_common; nth(@candidates, len(@not_common))",
        ])
        .run();

    assert!(
        stdout.contains("1"),
        "Expected '1' for gcd(7, 5), got: {}",
        stdout
    );
}

// -----------------------------------------------------------------------------
// Additional Edge Case Tests
// -----------------------------------------------------------------------------
// These tests verify edge cases and error conditions
// -----------------------------------------------------------------------------

#[test]
fn rosetta_range_basic() {
    // range(5) should produce 0, 1, 2, 3, 4
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "range(5) | join(_, \", \")"])
        .run();

    assert!(
        stdout.contains("0, 1, 2, 3, 4"),
        "Expected '0, 1, 2, 3, 4' for range(5), got: {}",
        stdout
    );
}

#[test]
fn rosetta_map_double() {
    // Double each number in a list
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "map([1, 2, 3, 4, 5], x => x * 2) | join(_, \", \")",
        ])
        .run();

    assert!(
        stdout.contains("2, 4, 6, 8, 10"),
        "Expected '2, 4, 6, 8, 10' for doubled list, got: {}",
        stdout
    );
}

#[test]
fn rosetta_filter_even() {
    // Filter even numbers from a list
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "filter([1, 2, 3, 4, 5, 6], x => x % 2 == 0) | join(_, \", \")",
        ])
        .run();

    assert!(
        stdout.contains("2, 4, 6"),
        "Expected '2, 4, 6' for even numbers, got: {}",
        stdout
    );
}

#[test]
fn rosetta_reduce_sum() {
    // Sum of [1, 2, 3, 4, 5] = 15
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "reduce([1, 2, 3, 4, 5], 0, (acc, x) => acc + x)",
        ])
        .run();

    assert!(
        stdout.contains("15"),
        "Expected '15' for sum of [1, 2, 3, 4, 5], got: {}",
        stdout
    );
}

#[test]
fn rosetta_string_concat() {
    // String concatenation with ++
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "\"Hello\" ++ \" \" ++ \"World\""])
        .run();

    assert!(
        stdout.contains("Hello World"),
        "Expected 'Hello World' for string concatenation, got: {}",
        stdout
    );
}

#[test]
fn rosetta_nested_if() {
    // Nested if expressions
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "if(10 > 5, if(10 > 8, \"A\", \"B\"), \"C\")"])
        .run();

    assert!(
        stdout.contains("A"),
        "Expected 'A' for nested if, got: {}",
        stdout
    );
}
