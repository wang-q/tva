#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use test_case::test_case;

// =============================================================================
// Expr Functions Documentation Tests
// =============================================================================
// This file contains tests for expr functions documented in docs/expr/functions.md
//
// Test organization follows the structure of functions.md:
// 1. Numeric Operations
// 2. String Manipulation
// 3. Generic Functions
// 4. Range Generation
// 5. List Operations
// 6. Logic & Control
// 7. Higher-Order Functions
// 8. Regular Expressions
// 9. Encoding & Hashing
// 10. Date & Time
// 11. IO
// 12. Meta Functions
//
// Last updated: 2026-03-19
// =============================================================================

// =============================================================================
// 1. Numeric Operations (functions.md#L7-L23)
// =============================================================================

// Single-argument numeric functions
#[test_case("abs(-42)", "42" ; "abs")]
#[test_case("ceil(3.14)", "4" ; "ceil")]
#[test_case("floor(3.14)", "3" ; "floor")]
#[test_case("round(3.5)", "4" ; "round")]
#[test_case("sqrt(16)", "4" ; "sqrt")]
#[test_case("ln(1)", "0" ; "ln")]
#[test_case("log10(100)", "2" ; "log10")]
#[test_case("exp(0)", "1" ; "exp")]
fn test_numeric_single_arg(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// Multi-argument numeric functions
#[test_case("pow(2, 10)", "1024" ; "pow")]
#[test_case("max(1, 5, 3, 9, 2)", "9" ; "max")]
#[test_case("min(1, 5, 3, -2, 2)", "-2" ; "min")]
fn test_numeric_multi_arg(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// Type conversion functions
#[test_case("int(\"42\")", "42" ; "int_conversion")]
#[test_case("float(\"3.14\")", "3.14" ; "float_conversion")]
fn test_numeric_type_conversion(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// Trigonometric functions
#[test_case("sin(0)", "0" ; "sin")]
#[test_case("cos(0)", "1" ; "cos")]
#[test_case("tan(0)", "0" ; "tan")]
fn test_numeric_trigonometric(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// =============================================================================
// 2. String Manipulation
// =============================================================================

// Basic string functions
#[test_case("trim(\"  hello  \")", "hello" ; "trim")]
#[test_case("upper(\"hello\")", "HELLO" ; "upper")]
#[test_case("lower(\"WORLD\")", "world" ; "lower")]
#[test_case("len(\"hello\")", "5" ; "len")]
#[test_case("char_len(\"你好\")", "2" ; "char_len")]
fn test_string_basic(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// String manipulation functions
#[test_case("substr(\"hello world\", 0, 5)", "hello" ; "substr")]
#[test_case("contains(\"hello\", \"ll\")", "true" ; "contains")]
#[test_case("starts_with(\"hello\", \"he\")", "true" ; "starts_with")]
#[test_case("ends_with(\"hello\", \"lo\")", "true" ; "ends_with")]
#[test_case("replace(\"hello\", \"l\", \"x\")", "hexxo" ; "replace")]
#[test_case("truncate(\"hello world\", 5)", "he..." ; "truncate")]
#[test_case("wordcount(\"hello world\")", "2" ; "wordcount")]
fn test_string_manipulation(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// String split and join
#[test_case("split(\"1,2,3\", \",\") | join(_, \"-\")", "1-2-3" ; "split_join")]
fn test_string_split_join(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// String formatting (fmt)
#[test_case("fmt(\"Hello %()!\", \"World\")", "Hello World!" ; "fmt_basic")]
#[test_case("fmt(\"%(1) has %(2) points\", \"Alice\", 100)", "Alice has 100 points" ; "fmt_positional")]
#[test_case("fmt(\"Hex: %(1:#x)\", 255)", "0xff" ; "fmt_hex")]
fn test_string_fmt(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// =============================================================================
// 3. Generic Functions
// =============================================================================

#[test_case("is_empty(\"\")", "true" ; "is_empty_string_true")]
#[test_case("is_empty(\"hello\")", "false" ; "is_empty_string_false")]
#[test_case("is_empty([])", "true" ; "is_empty_list_true")]
#[test_case("is_empty([1, 2, 3])", "false" ; "is_empty_list_false")]
#[test_case("take(\"hello\", 3)", "hel" ; "take_string")]
#[test_case("take([1, 2, 3, 4, 5], 3) | join(_, \", \")", "1, 2, 3" ; "take_list")]
#[test_case("drop(\"hello\", 2)", "llo" ; "drop_string")]
#[test_case("drop([1, 2, 3, 4, 5], 2) | join(_, \", \")", "3, 4, 5" ; "drop_list")]
#[test_case("concat(\"hello\", \" \", \"world\")", "hello world" ; "concat_strings_multi")]
#[test_case("concat([1, 2], [3, 4], [5, 6]) | join(_, \", \")", "1, 2, 3, 4, 5, 6" ; "concat_lists_multi")]
fn test_generic(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// =============================================================================
// 4. Range Generation
// =============================================================================

#[test_case("range(4) | join(_, \", \")", "0, 1, 2, 3" ; "range_single")]
#[test_case("range(2, 5) | join(_, \", \")", "2, 3, 4" ; "range_two")]
#[test_case("range(0, 10, 3) | join(_, \", \")", "0, 3, 6, 9" ; "range_three")]
#[test_case("range(0, -5, -1) | join(_, \", \")", "0, -1, -2, -3, -4" ; "range_negative_step")]
fn test_range(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// =============================================================================
// 5. List Operations
// =============================================================================

// Basic list functions
#[test_case("first([1, 2, 3])", "1" ; "first")]
#[test_case("last([1, 2, 3])", "3" ; "last")]
#[test_case("nth([10, 20, 30], 1)", "20" ; "nth")]
#[test_case("join([\"a\", \"b\", \"c\"], \"-\")", "a-b-c" ; "join")]
fn test_list_basic(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// List transformation functions
#[test_case("reverse([1, 2, 3]) | join(_, \", \")", "3, 2, 1" ; "reverse")]
#[test_case("replace_nth([1, 2, 3], 1, 99) | join(_, \", \")", "1, 99, 3" ; "replace_nth")]
#[test_case("slice([1, 2, 3, 4, 5], 1, 4) | join(_, \", \")", "2, 3, 4" ; "slice")]
#[test_case("sort([3, 1, 4, 1, 5]) | join(_, \", \")", "1, 1, 3, 4, 5" ; "sort")]
#[test_case("unique([1, 2, 2, 3, 3, 3]) | join(_, \", \")", "1, 2, 3" ; "unique")]
fn test_list_transformation(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// Advanced list functions from documentation
#[test_case("flatten([[1, 2], [3, 4]]) | join(_, \", \")", "1, 2, 3, 4" ; "flatten_nested")]
#[test_case("flatten([[1, 2], 3, [4, 5]]) | join(_, \", \")", "1, 2, 3, 4, 5" ; "flatten_mixed")]
#[test_case("zip([1, 2], [\"a\", \"b\"])", "[Int(1), String(\"a\")]" ; "zip_basic")]
#[test_case("grouped([1, 2, 3, 4], 2)", "[Int(1), Int(2)]" ; "grouped_even")]
#[test_case("partition([1, 2, 3, 4], x => x % 2 == 0) | first(_) | join(_, \", \")", "2, 4" ; "partition_even")]
#[test_case("flat_map([1, 2], x => [x, x * 2]) | join(_, \", \")", "1, 2, 2, 4" ; "flat_map_basic")]
fn test_list_advanced(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// =============================================================================
// 6. Logic & Control
// =============================================================================

#[test_case("if(true, \"yes\", \"no\")", "yes" ; "if_true")]
#[test_case("if(false, \"yes\", \"no\")", "no" ; "if_false")]
#[test_case("default(null, \"fallback\")", "fallback" ; "default_null")]
fn test_logic_control(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// =============================================================================
// 7. Higher-Order Functions
// =============================================================================

#[test_case("map([1, 2, 3], x => x * 2) | join(_, \", \")", "2, 4, 6" ; "map")]
#[test_case("filter([1, 2, 3, 4], x => x > 2) | join(_, \", \")", "3, 4" ; "filter")]
#[test_case("reduce([1, 2, 3], 0, (acc, x) => acc + x)", "6" ; "reduce_sum")]
#[test_case("reduce([\"a\", \"b\", \"c\"], 0, (acc, _) => acc + 1)", "3" ; "reduce_count")]
#[test_case("reduce([3, 1, 4, 1, 5], 0, (acc, x) => if(x > acc, x, acc))", "5" ; "reduce_max")]
#[test_case("sort_by([\"cherry\", \"apple\", \"pear\"], s => len(s)) | join(_, \", \")", "pear, apple, cherry" ; "sort_by_len")]
#[test_case("sort_by([-5, 3, -1, 4], x => abs(x)) | join(_, \", \")", "-1, 3, 4, -5" ; "sort_by_abs")]
#[test_case("sort_by([[3, \"c\"], [1, \"a\"], [2, \"b\"]], r => r.first())", "[Int(1), String(\"a\")]" ; "sort_by_first")]
#[test_case("sort_by([\"Banana\", \"apple\", \"Cherry\"], s => lower(s)) | join(_, \", \")", "apple, Banana, Cherry" ; "sort_by_lower")]
#[test_case("sort_by([[2, \"b\"], [1, \"c\"], [1, \"a\"]], r => [r.nth(0), r.nth(1)]) | nth(_, 0) | nth(_, 1)", "a" ; "sort_by_composite")]
#[test_case("take_while([1, 2, 3, 4, 5], x => x < 4) | join(_, \", \")", "1, 2, 3" ; "take_while_basic")]
#[test_case("take_while([2, 4, 6, 7, 8, 10], x => x % 2 == 0) | join(_, \", \")", "2, 4, 6" ; "take_while_even")]
#[test_case("take_while([\"apple\", \"apricot\", \"banana\"], s => s.starts_with(\"a\")) | join(_, \", \")", "apple, apricot" ; "take_while_string")]
#[test_case("filter_index([10, 15, 20, 25, 30], x => x > 18) | join(_, \", \")", "2, 3, 4" ; "filter_index_basic")]
#[test_case("filter_index([1, 2, 3, 4, 5], x => x % 2 == 0) | join(_, \", \")", "1, 3" ; "filter_index_even")]
#[test_case("concat([1, 2], [3, 4]) | join(_, \", \")", "1, 2, 3, 4" ; "concat_lists")]
#[test_case("concat(\"hello\", \" \", \"world\")", "hello world" ; "concat_strings")]
fn test_higher_order(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// =============================================================================
// 8. Regular Expressions
// =============================================================================

#[test_case("regex_match(\"hello\", \"h.*o\")", "true" ; "regex_match")]
#[test_case("regex_extract(\"hello world\", \"(\\w+)\", 1)", "hello" ; "regex_extract")]
#[test_case("regex_replace(\"hello 123\", \"\\d+\", \"XXX\")", "hello XXX" ; "regex_replace")]
fn test_regex(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// =============================================================================
// 9. Encoding & Hashing
// =============================================================================

#[test_case("md5(\"hello\")", "5d41402abc4b2a76b9719d911017c592" ; "md5")]
#[test_case("sha256(\"hello\")", "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824" ; "sha256")]
#[test_case("base64(\"hello\")", "aGVsbG8=" ; "base64_encode")]
#[test_case("unbase64(\"aGVsbG8=\")", "hello" ; "base64_decode")]
fn test_encoding_hash(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// =============================================================================
// 10. Date & Time
// =============================================================================

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

#[test_case("strftime(strptime(\"2024-03-15T00:00:00\", \"%Y-%m-%dT%H:%M:%S\"), \"%d/%m/%Y\")", "15/03/2024" ; "strptime_strftime")]
#[test_case("strftime(strptime(\"15/03/2024 14:30:00\", \"%d/%m/%Y %H:%M:%S\"), \"%Y-%m-%d\")", "2024-03-15" ; "strptime_alt_format")]
fn test_datetime_parse_format(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// =============================================================================
// 11. IO
// =============================================================================

#[test_case("print(\"hello\")", "hello" ; "print_basic")]
#[test_case("print(42)", "42" ; "print_number")]
fn test_io_print(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// =============================================================================
// 12. Meta Functions
// =============================================================================

#[test_case("type(42)", "int" ; "type_int")]
#[test_case("is_null(null)", "true" ; "is_null_true")]
#[test_case("is_null(\"hello\")", "false" ; "is_null_false")]
#[test_case("is_int(42)", "true" ; "is_int_true")]
#[test_case("is_int(3.14)", "false" ; "is_int_false")]
#[test_case("is_float(3.14)", "true" ; "is_float_true")]
#[test_case("is_numeric(42)", "true" ; "is_numeric_true")]
#[test_case("is_string(\"hello\")", "true" ; "is_string_true")]
#[test_case("is_bool(true)", "true" ; "is_bool_true")]
#[test_case("is_list([1, 2, 3])", "true" ; "is_list_true")]
fn test_meta(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
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

#[test]
fn test_meta_env() {
    // Test env function with PATH (should exist on all platforms)
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "env(\"PATH\")"]).run();
    assert!(stdout.len() > 0, "Expected PATH value, got: {}", stdout);
}
