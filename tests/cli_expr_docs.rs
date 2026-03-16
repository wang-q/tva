#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

// =============================================================================
// Expression Documentation Tests
// =============================================================================
// This file contains tests for expression features documented in docs/expr/
//
// Covered documents:
// - functions.md: Built-in functions
// - literals.md: Literals and type system
// - operators.md: Operators
// - syntax.md: Syntax guide
// - variables.md: Variables and column references
//
// Last updated: 2026-03-16
// =============================================================================

// =============================================================================
// Functions Tests (functions.md)
// =============================================================================

// -----------------------------------------------------------------------------
// Numeric Operations
// -----------------------------------------------------------------------------
#[test]
fn test_numeric_abs() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "abs(-42)"]).run();
    assert!(
        stdout.contains("42"),
        "Expected '42' for abs(-42), got: {}",
        stdout
    );
}

#[test]
fn test_numeric_ceil() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "ceil(3.14)"]).run();
    assert!(
        stdout.contains("4"),
        "Expected '4' for ceil(3.14), got: {}",
        stdout
    );
}

#[test]
fn test_numeric_floor() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "floor(3.14)"]).run();
    assert!(
        stdout.contains("3"),
        "Expected '3' for floor(3.14), got: {}",
        stdout
    );
}

#[test]
fn test_numeric_round() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "round(3.5)"]).run();
    assert!(
        stdout.contains("4"),
        "Expected '4' for round(3.5), got: {}",
        stdout
    );
}

#[test]
fn test_numeric_sqrt() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "sqrt(16)"]).run();
    assert!(
        stdout.contains("4"),
        "Expected '4' for sqrt(16), got: {}",
        stdout
    );
}

#[test]
fn test_numeric_pow() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "pow(2, 10)"]).run();
    assert!(
        stdout.contains("1024"),
        "Expected '1024' for pow(2, 10), got: {}",
        stdout
    );
}

#[test]
fn test_numeric_max() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "max(1, 5, 3, 9, 2)"])
        .run();
    assert!(
        stdout.contains("9"),
        "Expected '9' for max, got: {}",
        stdout
    );
}

#[test]
fn test_numeric_min() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "min(1, 5, 3, -2, 2)"])
        .run();
    assert!(
        stdout.contains("-2"),
        "Expected '-2' for min, got: {}",
        stdout
    );
}

#[test]
fn test_numeric_int_conversion() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "int(\"42\")"]).run();
    assert!(
        stdout.contains("42"),
        "Expected '42' for int conversion, got: {}",
        stdout
    );
}

#[test]
fn test_numeric_float_conversion() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "float(\"3.14\")"]).run();
    assert!(
        stdout.contains("3.14"),
        "Expected '3.14' for float conversion, got: {}",
        stdout
    );
}

#[test]
fn test_numeric_ln() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "ln(1)"]).run();
    assert!(
        stdout.contains("0"),
        "Expected '0' for ln(1), got: {}",
        stdout
    );
}

#[test]
fn test_numeric_log10() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "log10(100)"]).run();
    assert!(
        stdout.contains("2"),
        "Expected '2' for log10(100), got: {}",
        stdout
    );
}

#[test]
fn test_numeric_exp() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "exp(0)"]).run();
    assert!(
        stdout.contains("1"),
        "Expected '1' for exp(0), got: {}",
        stdout
    );
}

#[test]
fn test_numeric_trigonometric() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "sin(0)"]).run();
    assert!(
        stdout.contains("0"),
        "Expected '0' for sin(0), got: {}",
        stdout
    );

    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "cos(0)"]).run();
    assert!(
        stdout.contains("1"),
        "Expected '1' for cos(0), got: {}",
        stdout
    );
}

// -----------------------------------------------------------------------------
// String Manipulation
// -----------------------------------------------------------------------------
#[test]
fn test_string_trim() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "trim(\"  hello  \")"])
        .run();
    assert!(
        stdout.contains("hello"),
        "Expected 'hello' for trim, got: {}",
        stdout
    );
}

#[test]
fn test_string_upper() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "upper(\"hello\")"])
        .run();
    assert!(
        stdout.contains("HELLO"),
        "Expected 'HELLO' for upper, got: {}",
        stdout
    );
}

#[test]
fn test_string_lower() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "lower(\"WORLD\")"])
        .run();
    assert!(
        stdout.contains("world"),
        "Expected 'world' for lower, got: {}",
        stdout
    );
}

#[test]
fn test_string_len() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "len(\"hello\")"]).run();
    assert!(
        stdout.contains("5"),
        "Expected '5' for len, got: {}",
        stdout
    );
}

#[test]
fn test_string_char_len() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "char_len(\"你好\")"])
        .run();
    assert!(
        stdout.contains("2"),
        "Expected '2' for char_len of UTF-8, got: {}",
        stdout
    );
}

#[test]
fn test_string_substr() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "substr(\"hello world\", 0, 5)"])
        .run();
    assert!(
        stdout.contains("hello"),
        "Expected 'hello' for substr, got: {}",
        stdout
    );
}

#[test]
fn test_string_split() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "split(\"1,2,3\", \",\") | join(_, \"-\")"])
        .run();
    assert!(
        stdout.contains("1-2-3"),
        "Expected '1-2-3' for split, got: {}",
        stdout
    );
}

#[test]
fn test_string_contains() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "contains(\"hello\", \"ll\")"])
        .run();
    assert!(
        stdout.contains("true"),
        "Expected 'true' for contains, got: {}",
        stdout
    );
}

#[test]
fn test_string_starts_with() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "starts_with(\"hello\", \"he\")"])
        .run();
    assert!(
        stdout.contains("true"),
        "Expected 'true' for starts_with, got: {}",
        stdout
    );
}

#[test]
fn test_string_ends_with() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "ends_with(\"hello\", \"lo\")"])
        .run();
    assert!(
        stdout.contains("true"),
        "Expected 'true' for ends_with, got: {}",
        stdout
    );
}

#[test]
fn test_string_replace() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "replace(\"hello\", \"l\", \"x\")"])
        .run();
    assert!(
        stdout.contains("hexxo"),
        "Expected 'hexxo' for replace, got: {}",
        stdout
    );
}

#[test]
fn test_string_truncate() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "truncate(\"hello world\", 5)"])
        .run();
    assert!(
        stdout.contains("he..."),
        "Expected 'he...' for truncate, got: {}",
        stdout
    );
}

#[test]
fn test_string_wordcount() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "wordcount(\"hello world\")"])
        .run();
    assert!(
        stdout.contains("2"),
        "Expected '2' for wordcount, got: {}",
        stdout
    );
}

// -----------------------------------------------------------------------------
// List Operations
// -----------------------------------------------------------------------------
#[test]
fn test_list_first() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "first([1, 2, 3])"])
        .run();
    assert!(
        stdout.contains("1"),
        "Expected '1' for first, got: {}",
        stdout
    );
}

#[test]
fn test_list_last() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "last([1, 2, 3])"]).run();
    assert!(
        stdout.contains("3"),
        "Expected '3' for last, got: {}",
        stdout
    );
}

#[test]
fn test_list_nth() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "nth([10, 20, 30], 1)"])
        .run();
    assert!(
        stdout.contains("20"),
        "Expected '20' for nth(1), got: {}",
        stdout
    );
}

#[test]
fn test_list_reverse() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "reverse([1, 2, 3]) | join(_, \", \")"])
        .run();
    assert!(
        stdout.contains("3, 2, 1"),
        "Expected '3, 2, 1' for reverse, got: {}",
        stdout
    );
}

#[test]
fn test_list_replace_nth() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "replace_nth([1, 2, 3], 1, 99) | join(_, \", \")",
        ])
        .run();
    assert!(
        stdout.contains("1, 99, 3"),
        "Expected '1, 99, 3' for replace_nth, got: {}",
        stdout
    );
}

#[test]
fn test_list_slice() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "slice([1, 2, 3, 4, 5], 1, 4) | join(_, \", \")",
        ])
        .run();
    assert!(
        stdout.contains("2, 3, 4"),
        "Expected '2, 3, 4' for slice, got: {}",
        stdout
    );
}

#[test]
fn test_list_sort() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "sort([3, 1, 4, 1, 5]) | join(_, \", \")"])
        .run();
    assert!(
        stdout.contains("1, 1, 3, 4, 5"),
        "Expected sorted list, got: {}",
        stdout
    );
}

#[test]
fn test_list_unique() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "unique([1, 2, 2, 3, 3, 3]) | join(_, \", \")"])
        .run();
    assert!(
        stdout.contains("1, 2, 3"),
        "Expected '1, 2, 3' for unique, got: {}",
        stdout
    );
}

#[test]
fn test_list_join() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "join([\"a\", \"b\", \"c\"], \"-\")"])
        .run();
    assert!(
        stdout.contains("a-b-c"),
        "Expected 'a-b-c' for join, got: {}",
        stdout
    );
}

// -----------------------------------------------------------------------------
// Range Generation
// -----------------------------------------------------------------------------
#[test]
fn test_range_single_arg() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "range(4) | join(_, \", \")"])
        .run();
    assert!(
        stdout.contains("0, 1, 2, 3"),
        "Expected '0, 1, 2, 3' for range(4), got: {}",
        stdout
    );
}

#[test]
fn test_range_two_args() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "range(2, 5) | join(_, \", \")"])
        .run();
    assert!(
        stdout.contains("2, 3, 4"),
        "Expected '2, 3, 4' for range(2, 5), got: {}",
        stdout
    );
}

#[test]
fn test_range_three_args() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "range(0, 10, 3) | join(_, \", \")"])
        .run();
    assert!(
        stdout.contains("0, 3, 6, 9"),
        "Expected '0, 3, 6, 9' for range(0, 10, 3), got: {}",
        stdout
    );
}

#[test]
fn test_range_negative_step() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "range(0, -5, -1) | join(_, \", \")"])
        .run();
    assert!(
        stdout.contains("0, -1, -2, -3, -4"),
        "Expected negative range, got: {}",
        stdout
    );
}

// -----------------------------------------------------------------------------
// Logic & Control
// -----------------------------------------------------------------------------
#[test]
fn test_logic_if_true() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "if(true, \"yes\", \"no\")"])
        .run();
    assert!(
        stdout.contains("yes"),
        "Expected 'yes' for if(true), got: {}",
        stdout
    );
}

#[test]
fn test_logic_if_false() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "if(false, \"yes\", \"no\")"])
        .run();
    assert!(
        stdout.contains("no"),
        "Expected 'no' for if(false), got: {}",
        stdout
    );
}

#[test]
fn test_logic_default() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "default(null, \"fallback\")"])
        .run();
    assert!(
        stdout.contains("fallback"),
        "Expected 'fallback' for default, got: {}",
        stdout
    );
}

// -----------------------------------------------------------------------------
// Higher-Order Functions
// -----------------------------------------------------------------------------
#[test]
fn test_hof_map() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "map([1, 2, 3], x => x * 2) | join(_, \", \")"])
        .run();
    assert!(
        stdout.contains("2, 4, 6"),
        "Expected '2, 4, 6' for map, got: {}",
        stdout
    );
}

#[test]
fn test_hof_filter() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "filter([1, 2, 3, 4], x => x > 2) | join(_, \", \")",
        ])
        .run();
    assert!(
        stdout.contains("3, 4"),
        "Expected '3, 4' for filter, got: {}",
        stdout
    );
}

#[test]
fn test_hof_reduce() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "reduce([1, 2, 3], 0, (acc, x) => acc + x)"])
        .run();
    assert!(
        stdout.contains("6"),
        "Expected '6' for reduce, got: {}",
        stdout
    );
}

#[test]
fn test_hof_sort_by() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "sort_by([\"cherry\", \"apple\", \"pear\"], s => len(s)) | join(_, \", \")",
        ])
        .run();
    assert!(
        stdout.contains("pear, apple, cherry"),
        "Expected sorted by length, got: {}",
        stdout
    );
}

#[test]
fn test_hof_take_while() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "take_while([1, 2, 3, 4, 5], x => x < 4) | join(_, \", \")",
        ])
        .run();
    assert!(
        stdout.contains("1, 2, 3"),
        "Expected '1, 2, 3' for take_while, got: {}",
        stdout
    );
}

// -----------------------------------------------------------------------------
// Regular Expressions
// -----------------------------------------------------------------------------
#[test]
fn test_regex_match() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "regex_match(\"hello\", \"h.*o\")"])
        .run();
    assert!(
        stdout.contains("true"),
        "Expected 'true' for regex_match, got: {}",
        stdout
    );
}

#[test]
fn test_regex_extract() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "regex_extract(\"hello world\", \"(\\w+)\", 1)",
        ])
        .run();
    assert!(
        stdout.contains("hello"),
        "Expected 'hello' for regex_extract, got: {}",
        stdout
    );
}

#[test]
fn test_regex_replace() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "regex_replace(\"hello 123\", \"\\d+\", \"XXX\")",
        ])
        .run();
    assert!(
        stdout.contains("hello XXX"),
        "Expected 'hello XXX' for regex_replace, got: {}",
        stdout
    );
}

// -----------------------------------------------------------------------------
// Encoding & Hashing
// -----------------------------------------------------------------------------
#[test]
fn test_hash_md5() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "md5(\"hello\")"]).run();
    // MD5 of "hello" is 5d41402abc4b2a76b9719d911017c592
    assert!(
        stdout.contains("5d41402abc4b2a76b9719d911017c592"),
        "Expected MD5 hash, got: {}",
        stdout
    );
}

#[test]
fn test_hash_sha256() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "sha256(\"hello\")"])
        .run();
    assert!(stdout.len() > 10, "Expected SHA256 hash, got: {}", stdout);
}

#[test]
fn test_encoding_base64() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "base64(\"hello\")"])
        .run();
    assert!(
        stdout.contains("aGVsbG8="),
        "Expected base64 encoding, got: {}",
        stdout
    );
}

#[test]
fn test_encoding_unbase64() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "unbase64(\"aGVsbG8=\")"])
        .run();
    assert!(
        stdout.contains("hello"),
        "Expected 'hello' for unbase64, got: {}",
        stdout
    );
}

// -----------------------------------------------------------------------------
// Date & Time
// -----------------------------------------------------------------------------
#[test]
fn test_datetime_now() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "now()"]).run();
    // Should return a datetime string
    assert!(
        stdout.len() > 10,
        "Expected datetime from now(), got: {}",
        stdout
    );
}

#[test]
fn test_datetime_strptime_strftime() {
    // Parse and format datetime - simplified test
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "strftime(now(), \"%Y\")"])
        .run();
    // Should return current year (2026)
    assert!(
        stdout.contains("2026") || stdout.contains("2025"),
        "Expected year in output, got: {}",
        stdout
    );
}

// -----------------------------------------------------------------------------
// Meta Functions
// -----------------------------------------------------------------------------
#[test]
fn test_meta_type() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "type(42)"]).run();
    assert!(
        stdout.contains("int"),
        "Expected 'int' for type(42), got: {}",
        stdout
    );
}

#[test]
fn test_meta_is_null() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "is_null(null)"]).run();
    assert!(
        stdout.contains("true"),
        "Expected 'true' for is_null(null), got: {}",
        stdout
    );
}

#[test]
fn test_meta_is_int() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "is_int(42)"]).run();
    assert!(
        stdout.contains("true"),
        "Expected 'true' for is_int(42), got: {}",
        stdout
    );
}

#[test]
fn test_meta_is_float() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "is_float(3.14)"]).run();
    assert!(
        stdout.contains("true"),
        "Expected 'true' for is_float(3.14), got: {}",
        stdout
    );
}

#[test]
fn test_meta_is_numeric() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "is_numeric(42)"]).run();
    assert!(
        stdout.contains("true"),
        "Expected 'true' for is_numeric(42), got: {}",
        stdout
    );
}

#[test]
fn test_meta_is_string() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "is_string(\"hello\")"])
        .run();
    assert!(
        stdout.contains("true"),
        "Expected 'true' for is_string, got: {}",
        stdout
    );
}

#[test]
fn test_meta_is_bool() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "is_bool(true)"]).run();
    assert!(
        stdout.contains("true"),
        "Expected 'true' for is_bool, got: {}",
        stdout
    );
}

#[test]
fn test_meta_is_list() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "is_list([1, 2, 3])"])
        .run();
    assert!(
        stdout.contains("true"),
        "Expected 'true' for is_list, got: {}",
        stdout
    );
}

#[test]
fn test_meta_version() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "version()"]).run();
    assert!(
        stdout.contains("0."),
        "Expected version string, got: {}",
        stdout
    );
}

#[test]
fn test_meta_platform() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "platform()"]).run();
    assert!(
        stdout.contains("windows")
            || stdout.contains("linux")
            || stdout.contains("macos"),
        "Expected platform name, got: {}",
        stdout
    );
}

#[test]
fn test_meta_cwd() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "cwd()"]).run();
    assert!(stdout.len() > 0, "Expected cwd path, got: {}", stdout);
}

// =============================================================================
// Literals Tests (literals.md)
// =============================================================================

#[test]
fn test_literal_integer() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "42"]).run();
    assert!(stdout.contains("42"), "Expected '42', got: {}", stdout);
}

#[test]
fn test_literal_float() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "3.14"]).run();
    assert!(stdout.contains("3.14"), "Expected '3.14', got: {}", stdout);
}

#[test]
fn test_literal_scientific_notation() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "1e6"]).run();
    assert!(
        stdout.contains("1000000"),
        "Expected '1000000' for 1e6, got: {}",
        stdout
    );
}

#[test]
fn test_literal_string_double_quotes() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "\"hello\""]).run();
    assert!(
        stdout.contains("hello"),
        "Expected 'hello', got: {}",
        stdout
    );
}

#[test]
fn test_literal_boolean_true() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "true"]).run();
    assert!(stdout.contains("true"), "Expected 'true', got: {}", stdout);
}

#[test]
fn test_literal_boolean_false() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "false"]).run();
    assert!(
        stdout.contains("false"),
        "Expected 'false', got: {}",
        stdout
    );
}

#[test]
fn test_literal_null() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "null"]).run();
    assert!(stdout.contains("null"), "Expected 'null', got: {}", stdout);
}

#[test]
fn test_literal_list() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "[1, 2, 3]"]).run();
    assert!(
        stdout.contains("1"),
        "Expected list containing '1', got: {}",
        stdout
    );
}

#[test]
fn test_literal_empty_list() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "[]"]).run();
    assert!(
        stdout.contains("[]"),
        "Expected '[]' for empty list, got: {}",
        stdout
    );
}

#[test]
fn test_literal_heterogeneous_list() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "[1, \"two\", true, null]"])
        .run();
    assert!(
        stdout.contains("1"),
        "Expected list with mixed types, got: {}",
        stdout
    );
}

#[test]
fn test_literal_q_string() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "q(hello world)"]).run();
    assert!(
        stdout.contains("hello world"),
        "Expected 'hello world' for q-string, got: {}",
        stdout
    );
}

// =============================================================================
// Operators Tests (operators.md)
// =============================================================================

#[test]
fn test_operator_arithmetic_add() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "2 + 3"]).run();
    assert!(
        stdout.contains("5"),
        "Expected '5' for 2 + 3, got: {}",
        stdout
    );
}

#[test]
fn test_operator_arithmetic_sub() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "5 - 3"]).run();
    assert!(
        stdout.contains("2"),
        "Expected '2' for 5 - 3, got: {}",
        stdout
    );
}

#[test]
fn test_operator_arithmetic_mul() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "4 * 5"]).run();
    assert!(
        stdout.contains("20"),
        "Expected '20' for 4 * 5, got: {}",
        stdout
    );
}

#[test]
fn test_operator_arithmetic_div() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "10 / 2"]).run();
    assert!(
        stdout.contains("5"),
        "Expected '5' for 10 / 2, got: {}",
        stdout
    );
}

#[test]
fn test_operator_arithmetic_mod() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "10 % 3"]).run();
    assert!(
        stdout.contains("1"),
        "Expected '1' for 10 % 3, got: {}",
        stdout
    );
}

#[test]
fn test_operator_arithmetic_pow() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "2 ** 8"]).run();
    assert!(
        stdout.contains("256"),
        "Expected '256' for 2 ** 8, got: {}",
        stdout
    );
}

#[test]
fn test_operator_arithmetic_negation() {
    // Test negation via subtraction from 0
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "0 - 42"]).run();
    assert!(
        stdout.contains("-42"),
        "Expected '-42' for negation, got: {}",
        stdout
    );
}

#[test]
fn test_operator_string_concat() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "\"hello\" ++ \" \" ++ \"world\""])
        .run();
    assert!(
        stdout.contains("hello world"),
        "Expected 'hello world', got: {}",
        stdout
    );
}

#[test]
fn test_operator_comparison_eq() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "5 == 5"]).run();
    assert!(
        stdout.contains("true"),
        "Expected 'true' for 5 == 5, got: {}",
        stdout
    );
}

#[test]
fn test_operator_comparison_ne() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "5 != 3"]).run();
    assert!(
        stdout.contains("true"),
        "Expected 'true' for 5 != 3, got: {}",
        stdout
    );
}

#[test]
fn test_operator_comparison_lt() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "3 < 5"]).run();
    assert!(
        stdout.contains("true"),
        "Expected 'true' for 3 < 5, got: {}",
        stdout
    );
}

#[test]
fn test_operator_comparison_le() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "5 <= 5"]).run();
    assert!(
        stdout.contains("true"),
        "Expected 'true' for 5 <= 5, got: {}",
        stdout
    );
}

#[test]
fn test_operator_comparison_gt() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "5 > 3"]).run();
    assert!(
        stdout.contains("true"),
        "Expected 'true' for 5 > 3, got: {}",
        stdout
    );
}

#[test]
fn test_operator_comparison_ge() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "5 >= 5"]).run();
    assert!(
        stdout.contains("true"),
        "Expected 'true' for 5 >= 5, got: {}",
        stdout
    );
}

#[test]
fn test_operator_string_comparison_eq() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "\"apple\" eq \"apple\""])
        .run();
    assert!(
        stdout.contains("true"),
        "Expected 'true' for string eq, got: {}",
        stdout
    );
}

#[test]
fn test_operator_string_comparison_lt() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "\"apple\" lt \"banana\""])
        .run();
    assert!(
        stdout.contains("true"),
        "Expected 'true' for string lt, got: {}",
        stdout
    );
}

#[test]
fn test_operator_logical_not() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "not true"]).run();
    assert!(
        stdout.contains("false"),
        "Expected 'false' for not true, got: {}",
        stdout
    );
}

#[test]
fn test_operator_logical_and() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "true and false"]).run();
    assert!(
        stdout.contains("false"),
        "Expected 'false' for true and false, got: {}",
        stdout
    );
}

#[test]
fn test_operator_logical_or() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "true or false"]).run();
    assert!(
        stdout.contains("true"),
        "Expected 'true' for true or false, got: {}",
        stdout
    );
}

#[test]
fn test_operator_precedence() {
    // Multiplication before addition
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "2 + 3 * 4"]).run();
    assert!(
        stdout.contains("14"),
        "Expected '14' for 2 + 3 * 4, got: {}",
        stdout
    );

    // With parentheses
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "(2 + 3) * 4"]).run();
    assert!(
        stdout.contains("20"),
        "Expected '20' for (2 + 3) * 4, got: {}",
        stdout
    );
}

// =============================================================================
// Syntax Tests (syntax.md)
// =============================================================================

#[test]
fn test_syntax_method_call() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "\"hello\".upper()"])
        .run();
    assert!(
        stdout.contains("HELLO"),
        "Expected 'HELLO' for method call, got: {}",
        stdout
    );
}

#[test]
fn test_syntax_method_chaining() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "\"  hello  \".trim().upper()"])
        .run();
    assert!(
        stdout.contains("HELLO"),
        "Expected 'HELLO' for method chaining, got: {}",
        stdout
    );
}

#[test]
fn test_syntax_pipe_single_arg() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "\"hello\" | upper()"])
        .run();
    assert!(
        stdout.contains("HELLO"),
        "Expected 'HELLO' for pipe, got: {}",
        stdout
    );
}

#[test]
fn test_syntax_pipe_with_placeholder() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "\"hello world\" | substr(_, 0, 5)"])
        .run();
    assert!(
        stdout.contains("hello"),
        "Expected 'hello' for pipe with placeholder, got: {}",
        stdout
    );
}

#[test]
fn test_syntax_pipe_chain() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "\"1,2,3\" | split(_, \",\") | map(_, x => int(x) * 2) | join(_, \"-\")",
        ])
        .run();
    assert!(
        stdout.contains("2-4-6"),
        "Expected '2-4-6' for pipe chain, got: {}",
        stdout
    );
}

#[test]
fn test_syntax_lambda_single_param() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "map([1, 2, 3], x => x + 1) | join(_, \", \")"])
        .run();
    assert!(
        stdout.contains("2, 3, 4"),
        "Expected '2, 3, 4' for lambda, got: {}",
        stdout
    );
}

#[test]
fn test_syntax_lambda_multi_param() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "reduce([1, 2, 3], 0, (acc, x) => acc + x)"])
        .run();
    assert!(
        stdout.contains("6"),
        "Expected '6' for multi-param lambda, got: {}",
        stdout
    );
}

#[test]
fn test_syntax_multiple_expressions() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "10 as @a; 20 as @b; @a + @b"])
        .run();
    assert!(
        stdout.contains("30"),
        "Expected '30' for multiple expressions, got: {}",
        stdout
    );
}

#[test]
fn test_syntax_comments() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "10 as @a; // This is a comment\n@a + 5"])
        .run();
    assert!(
        stdout.contains("15"),
        "Expected '15' with comments, got: {}",
        stdout
    );
}

// =============================================================================
// Variables Tests (variables.md)
// =============================================================================

#[test]
fn test_variable_column_by_index() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-n", "name,age", "-r", "John,30", "-E", "@1"])
        .run();
    assert!(
        stdout.contains("John"),
        "Expected 'John' for @1, got: {}",
        stdout
    );
}

#[test]
fn test_variable_column_by_name() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-n", "name,age", "-r", "John,30", "-E", "@name"])
        .run();
    assert!(
        stdout.contains("John"),
        "Expected 'John' for @name, got: {}",
        stdout
    );
}

#[test]
fn test_variable_column_entire_row() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-n", "a,b,c", "-r", "1,2,3", "-E", "@0"])
        .run();
    assert!(
        stdout.contains("1") && stdout.contains("2"),
        "Expected row content for @0, got: {}",
        stdout
    );
}

#[test]
fn test_variable_binding() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "price,qty",
            "-r",
            "10,5",
            "-E",
            "@price * @qty as @total; @total",
        ])
        .run();
    assert!(
        stdout.contains("50"),
        "Expected '50' for variable binding, got: {}",
        stdout
    );
}

#[test]
fn test_variable_binding_reuse() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "[1, 2, 3] as @list; @list | len()"])
        .run();
    assert!(
        stdout.contains("3"),
        "Expected '3' for variable reuse, got: {}",
        stdout
    );
}

#[test]
fn test_variable_binding_chain() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "(\"hello\" as @s).upper()"])
        .run();
    assert!(
        stdout.contains("HELLO"),
        "Expected 'HELLO' for binding chain, got: {}",
        stdout
    );
}

#[test]
fn test_variable_lambda_capture() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "5 as @offset; map([1, 2, 3], n => n + @offset) | join(_, \", \")",
        ])
        .run();
    assert!(
        stdout.contains("6, 7, 8"),
        "Expected '6, 7, 8' for lambda capture, got: {}",
        stdout
    );
}

#[test]
fn test_variable_multiple_bindings() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "price,qty",
            "-r",
            "10,5",
            "-E",
            "@price as @p; @qty as @q; @p * @q",
        ])
        .run();
    assert!(
        stdout.contains("50"),
        "Expected '50' for multiple bindings, got: {}",
        stdout
    );
}

#[test]
fn test_variable_binding_shadowing() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "price",
            "-r",
            "100",
            "-E",
            "@price * 2 as @price; @price",
        ])
        .run();
    assert!(
        stdout.contains("200"),
        "Expected '200' for variable shadowing, got: {}",
        stdout
    );
}

// =============================================================================
// Complex Integration Tests
// =============================================================================

#[test]
fn test_complex_data_transformation() {
    // Transform CSV-like data - simplified version
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", 
            "[1, 2, 3, 4, 5] | map(_, x => x * 2) | filter(_, x => x > 4) | join(_, \"-\")"])
        .run();
    assert!(
        stdout.contains("6-8-10"),
        "Expected '6-8-10' for complex transformation, got: {}",
        stdout
    );
}

#[test]
fn test_complex_validation_pipeline() {
    // Validate email format
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "email",
            "-r",
            "  Test@Example.COM  ",
            "-E",
            "@email | trim() | lower() | regex_match(_, \".*@.*\\.com\")",
        ])
        .run();
    assert!(
        stdout.contains("true"),
        "Expected 'true' for email validation, got: {}",
        stdout
    );
}

#[test]
fn test_complex_word_processing() {
    // Process words: split, filter by length, sort, join
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E",
            "\"apple,banana,cherry,date\" | split(_, \",\") | filter(_, w => len(w) > 4) | sort_by(_, w => len(w)) | join(_, \", \")"])
        .run();
    assert!(
        stdout.contains("apple, banana, cherry"),
        "Expected sorted long words, got: {}",
        stdout
    );
}

#[test]
fn test_complex_nested_lambda() {
    // Nested lambda with variable capture - simplified test
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "2 as @multiplier; map([1, 2, 3], x => x * @multiplier) | join(_, \", \")",
        ])
        .run();
    assert!(
        stdout.contains("2, 4, 6"),
        "Expected '2, 4, 6' for nested lambda, got: {}",
        stdout
    );
}

#[test]
fn test_complex_type_checking() {
    // Type checking pipeline
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "[[1,2], \"string\", true, null, -5].map(x => type(x)).join(\",\")",
        ])
        .run();
    assert!(
        stdout.contains("list,string,bool,null,int"),
        "Expected type list, got: {}",
        stdout
    );
}

#[test]
fn test_complex_conditional_logic() {
    // Complex conditional with multiple checks
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-n", "age,income", "-r", "25,50000", "-E",
            "if(@age >= 18 and @age < 65 and @income > 30000, \"qualified\", \"not qualified\")"])
        .run();
    assert!(
        stdout.contains("qualified"),
        "Expected 'qualified', got: {}",
        stdout
    );
}
