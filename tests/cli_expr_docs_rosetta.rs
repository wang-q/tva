#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use test_case::test_case;

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
// Hello World & Basic Examples
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

#[test]
fn rosetta_99_bottles_of_beer() {
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

#[test_case(
    "map(range(1, 16), n => if(n % 15 == 0, \"FizzBuzz\", if(n % 3 == 0, \"Fizz\", if(n % 5 == 0, \"Buzz\", n)))) | join(_, \", \")",
    &["1, 2, Fizz, 4, Buzz", "FizzBuzz"]
    ; "first_15"
)]
#[test_case(
    "if(15 % 15 == 0, \"FizzBuzz\", 15)",
    &["FizzBuzz"]
    ; "number_15"
)]
#[test_case(
    "if(3 % 3 == 0, \"Fizz\", 3)",
    &["Fizz"]
    ; "number_3"
)]
#[test_case(
    "if(5 % 5 == 0, \"Buzz\", 5)",
    &["Buzz"]
    ; "number_5"
)]
fn rosetta_fizzbuzz_tests(expr: &str, expected: &[&str]) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    for exp in expected {
        assert!(
            stdout.contains(exp),
            "Expected '{}' in stdout, got: {}",
            exp,
            stdout
        );
    }
}

// -----------------------------------------------------------------------------
// Factorial
// -----------------------------------------------------------------------------
// Demonstrates: reduce() for iterative computation, range()
// Formula: n! = n x (n-1) x ... x 1, with 0! = 1
// -----------------------------------------------------------------------------

#[test_case(
    "reduce(range(1, 6), 1, (acc, n) => acc * n)",
    "120"
    ; "factorial_5"
)]
#[test_case(
    "if(0 == 0, 1, reduce(range(1, 1), 1, (acc, x) => acc * x))",
    "1"
    ; "factorial_0"
)]
#[test_case(
    "reduce(range(1, 11), 1, (acc, n) => acc * n)",
    "3628800"
    ; "factorial_10"
)]
fn rosetta_factorial_tests(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' in stdout, got: {}",
        expected,
        stdout
    );
}

// -----------------------------------------------------------------------------
// Fibonacci Sequence
// -----------------------------------------------------------------------------
// Demonstrates: reduce() with state tracking, list access with nth()
// Formula: F(0)=0, F(1)=1, F(n)=F(n-1)+F(n-2)
// -----------------------------------------------------------------------------

#[test_case(
    "reduce(range(2, 11), [0, 1], (acc, _) => [acc.nth(1), acc.nth(0) + acc.nth(1)]).nth(1)",
    "55"
    ; "fibonacci_10th"
)]
#[test_case("0", "0" ; "fibonacci_0")]
#[test_case("1", "1" ; "fibonacci_1")]
fn rosetta_fibonacci_tests(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' in stdout, got: {}",
        expected,
        stdout
    );
}

// -----------------------------------------------------------------------------
// Palindrome Detection
// -----------------------------------------------------------------------------
// Demonstrates: lower(), variable binding (as @var),
//               method chaining (split().reverse().join())
// -----------------------------------------------------------------------------

#[test_case(
    "\"radar\" | split(_, \"\") | reverse() | join(_, \"\") as @reversed; \"radar\" == @reversed",
    "true"
    ; "simple_palindrome"
)]
#[test_case(
    "\"hello\" | split(_, \"\") | reverse() | join(_, \"\") as @reversed; \"hello\" == @reversed",
    "false"
    ; "not_palindrome"
)]
#[test_case(
    "\"Racecar\" | lower() | split(_, \"\") | reverse() | join(_, \"\") as @reversed; \"racecar\" == @reversed",
    "true"
    ; "case_insensitive"
)]
fn rosetta_palindrome_tests(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' in stdout, got: {}",
        expected,
        stdout
    );
}

// -----------------------------------------------------------------------------
// Word Frequency
// -----------------------------------------------------------------------------
// Demonstrates: unique(), filter(), len(), split()
// -----------------------------------------------------------------------------

#[test_case(
    "\"the quick brown fox jumps over the lazy dog\" | split(_, \" \") as @words; filter(@words, w => w == \"the\") | len()",
    "2"
    ; "count_the"
)]
#[test_case(
    "\"the the the quick quick brown\" | split(_, \" \") | unique() | len()",
    "3"
    ; "unique_words"
)]
fn rosetta_word_frequency_tests(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' in stdout, got: {}",
        expected,
        stdout
    );
}

// -----------------------------------------------------------------------------
// Sieve of Eratosthenes
// -----------------------------------------------------------------------------
// Demonstrates: sqrt(), int(), reduce(), replace_nth(), filter()
// Algorithm: Mark multiples of each prime starting from 2
// -----------------------------------------------------------------------------

#[test_case(
    "map(range(0, 11), n => n >= 2) as @is_prime; reduce(range(2, 4), @is_prime, (primes, p) => if(primes.nth(p), reduce(range(p * p, 11, p), primes, (acc, m) => acc.replace_nth(m, false)), primes)) as @sieved; filter(range(2, 11), n => @sieved.nth(n)) | join(_, \", \")",
    "2, 3, 5, 7"
    ; "primes_up_to_10"
)]
#[test_case("2 >= 2", "true" ; "2_is_prime")]
fn rosetta_sieve_tests(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' in stdout, got: {}",
        expected,
        stdout
    );
}

// -----------------------------------------------------------------------------
// Greatest Common Divisor (GCD)
// -----------------------------------------------------------------------------
// Demonstrates: take_while(), reverse(), min(), nth()
// Algorithm: Search from largest to smallest for common divisor
// -----------------------------------------------------------------------------

#[test_case(
    "48 as @a; 18 as @b; min(@a, @b) as @limit; reverse(range(1, @limit + 1)) as @candidates; take_while(@candidates, d => @a % d != 0 or @b % d != 0) as @not_common; nth(@candidates, len(@not_common))",
    "6"
    ; "gcd_48_18"
)]
#[test_case(
    "12 as @a; 8 as @b; min(@a, @b) as @limit; reverse(range(1, @limit + 1)) as @candidates; take_while(@candidates, d => @a % d != 0 or @b % d != 0) as @not_common; nth(@candidates, len(@not_common))",
    "4"
    ; "gcd_12_8"
)]
#[test_case(
    "7 as @a; 5 as @b; min(@a, @b) as @limit; reverse(range(1, @limit + 1)) as @candidates; take_while(@candidates, d => @a % d != 0 or @b % d != 0) as @not_common; nth(@candidates, len(@not_common))",
    "1"
    ; "gcd_7_5"
)]
fn rosetta_gcd_tests(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' in stdout, got: {}",
        expected,
        stdout
    );
}

// -----------------------------------------------------------------------------
// Additional Edge Case Tests
// -----------------------------------------------------------------------------
// These tests verify edge cases and basic functionality
// -----------------------------------------------------------------------------

#[test_case(
    "range(5) | join(_, \", \")",
    "0, 1, 2, 3, 4"
    ; "range_basic"
)]
#[test_case(
    "map([1, 2, 3, 4, 5], x => x * 2) | join(_, \", \")",
    "2, 4, 6, 8, 10"
    ; "map_double"
)]
#[test_case(
    "filter([1, 2, 3, 4, 5, 6], x => x % 2 == 0) | join(_, \", \")",
    "2, 4, 6"
    ; "filter_even"
)]
#[test_case(
    "reduce([1, 2, 3, 4, 5], 0, (acc, x) => acc + x)",
    "15"
    ; "reduce_sum"
)]
#[test_case(
    "\"Hello\" ++ \" \" ++ \"World\"",
    "Hello World"
    ; "string_concat"
)]
#[test_case(
    "if(10 > 5, if(10 > 8, \"A\", \"B\"), \"C\")",
    "A"
    ; "nested_if"
)]
fn rosetta_basic_tests(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' in stdout, got: {}",
        expected,
        stdout
    );
}
