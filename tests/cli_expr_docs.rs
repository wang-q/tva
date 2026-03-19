#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use test_case::test_case;

// =============================================================================
// Expr Documentation Tests
// =============================================================================
// This file contains tests for expr features documented in docs/expr/
//
// Covered documents:
// - functions.md: Built-in functions (moved to cli_expr_docs_functions.rs)
// - literals.md: Literals and type system
// - operators.md: Operators
// - syntax.md: Syntax guide
// - variables.md: Variables and column references
//
// Last updated: 2026-03-19
// =============================================================================

// =============================================================================
// Literals Tests (literals.md)
// =============================================================================

#[test_case("42", "42" ; "literal_integer")]
#[test_case("3.14", "3.14" ; "literal_float")]
#[test_case("1e6", "1000000" ; "literal_scientific")]
#[test_case("\"hello\"", "hello" ; "literal_string")]
#[test_case("true", "true" ; "literal_true")]
#[test_case("false", "false" ; "literal_false")]
#[test_case("null", "null" ; "literal_null")]
#[test_case("[]", "[]" ; "literal_empty_list")]
fn test_literals(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

#[test_case("[1, 2, 3]", "1" ; "literal_list")]
#[test_case("[1, \"two\", true, null]", "1" ; "literal_heterogeneous")]
fn test_literals_contains(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' in output for {}, got: {}",
        expected,
        expr,
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

// Arithmetic operators
#[test_case("2 + 3", "5" ; "add")]
#[test_case("5 - 3", "2" ; "sub")]
#[test_case("4 * 5", "20" ; "mul")]
#[test_case("10 / 2", "5" ; "div")]
#[test_case("10 % 3", "1" ; "modulo")]
#[test_case("2 ** 8", "256" ; "pow")]
#[test_case("0 - 42", "-42" ; "negation")]
fn test_operator_arithmetic(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// String concatenation
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

// Comparison operators
#[test_case("5 == 5", "true" ; "eq")]
#[test_case("5 != 3", "true" ; "ne")]
#[test_case("3 < 5", "true" ; "lt")]
#[test_case("5 <= 5", "true" ; "le")]
#[test_case("5 > 3", "true" ; "gt")]
#[test_case("5 >= 5", "true" ; "ge")]
fn test_operator_comparison(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// String comparison operators
#[test_case("\"apple\" eq \"apple\"", "true" ; "str_eq")]
#[test_case("\"apple\" lt \"banana\"", "true" ; "str_lt")]
fn test_operator_string_comparison(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// Logical operators
#[test_case("not true", "false" ; "not")]
#[test_case("true and false", "false" ; "and")]
#[test_case("true or false", "true" ; "or")]
fn test_operator_logical(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// Operator precedence
#[test_case("2 + 3 * 4", "14" ; "precedence_mul_before_add")]
#[test_case("(2 + 3) * 4", "20" ; "precedence_parens")]
fn test_operator_precedence(expr: &str, expected: &str) {
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
// Syntax Tests (syntax.md)
// =============================================================================

// Method calls
#[test_case("\"hello\".upper()", "HELLO" ; "method_call")]
#[test_case("\"  hello  \".trim().upper()", "HELLO" ; "method_chaining")]
fn test_syntax_method(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// Pipe syntax
#[test_case("\"hello\" | upper()", "HELLO" ; "pipe_single")]
#[test_case("\"hello world\" | substr(_, 0, 5)", "hello" ; "pipe_placeholder")]
fn test_syntax_pipe(expr: &str, expected: &str) {
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

// Lambda expressions
#[test_case("map([1, 2, 3], x => x + 1) | join(_, \", \")", "2, 3, 4" ; "lambda_single")]
#[test_case("reduce([1, 2, 3], 0, (acc, x) => acc + x)", "6" ; "lambda_multi")]
fn test_syntax_lambda(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// Multiple expressions and comments
#[test_case("10 as @a; 20 as @b; @a + @b", "30" ; "multiple_expressions")]
#[test_case("10 as @a; // comment\n@a + 5", "15" ; "comments")]
fn test_syntax_expressions(expr: &str, expected: &str) {
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
// Variables Tests (variables.md)
// =============================================================================

// Column references
#[test_case("@1", "John" ; "column_by_index")]
#[test_case("@name", "John" ; "column_by_name")]
fn test_variable_column_reference(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-n", "name,age", "-r", "John,30", "-E", expr])
        .run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
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

// Variable binding
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

#[test_case("[1, 2, 3] as @list; @list | len()", "3" ; "binding_reuse")]
#[test_case("(\"hello\" as @s).upper()", "HELLO" ; "binding_chain")]
fn test_variable_binding_simple(expr: &str, expected: &str) {
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
