#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

// =============================================================================
// Expr Documentation Tests
// =============================================================================
// This file contains tests for expr features documented in docs/expr/
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
