//! Basic integration tests for expression engine

use crate::libs::expr::eval_expr;

// Helper to create a test row
fn row(values: &[&str]) -> Vec<String> {
    values.iter().map(|s| s.to_string()).collect()
}

// Helper to create headers
fn headers(names: &[&str]) -> Vec<String> {
    names.iter().map(|s| s.to_string()).collect()
}

#[test]
fn test_simple_column_reference() {
    let r = row(&["10", "20", "30"]);

    assert_eq!(eval_expr("@1", &r, None).unwrap().to_string(), "10");
    assert_eq!(eval_expr("@2", &r, None).unwrap().to_string(), "20");
    assert_eq!(eval_expr("@3", &r, None).unwrap().to_string(), "30");
}

#[test]
fn test_basic_arithmetic() {
    let r = row(&["10", "5"]);

    assert_eq!(eval_expr("@1 + @2", &r, None).unwrap().to_string(), "15");
    assert_eq!(eval_expr("@1 - @2", &r, None).unwrap().to_string(), "5");
    assert_eq!(eval_expr("@1 * @2", &r, None).unwrap().to_string(), "50");
    assert_eq!(eval_expr("@1 / @2", &r, None).unwrap().to_string(), "2");
}

#[test]
fn test_operator_precedence() {
    let r = row(&["10", "3", "2"]);

    // Multiplication before addition: 10 + 3 * 2 = 16
    assert_eq!(
        eval_expr("@1 + @2 * @3", &r, None).unwrap().to_string(),
        "16"
    );

    // Parentheses override: (10 + 3) * 2 = 26
    assert_eq!(
        eval_expr("(@1 + @2) * @3", &r, None).unwrap().to_string(),
        "26"
    );
}

#[test]
fn test_column_reference_by_name() {
    let h = headers(&["price", "quantity"]);
    let r = row(&["100", "5"]);

    assert_eq!(
        eval_expr("@price", &r, Some(&h)).unwrap().to_string(),
        "100"
    );
    assert_eq!(
        eval_expr("@quantity", &r, Some(&h)).unwrap().to_string(),
        "5"
    );
    assert_eq!(
        eval_expr("@price * @quantity", &r, Some(&h))
            .unwrap()
            .to_string(),
        "500"
    );
}

#[test]
fn test_number_literals() {
    let r: Vec<String> = vec![];

    assert_eq!(eval_expr("42", &r, None).unwrap().to_string(), "42");
    assert_eq!(eval_expr("3.14", &r, None).unwrap().to_string(), "3.14");
}

#[test]
fn test_mixed_types() {
    let r = row(&["10", "3.5"]);

    let result = eval_expr("@1 + @2", &r, None).unwrap();
    assert!(result.to_string().starts_with("13.5"));
}

#[test]
fn test_string_literals() {
    let r: Vec<String> = vec![];

    assert_eq!(
        eval_expr("\"hello\"", &r, None).unwrap().to_string(),
        "hello"
    );
    assert_eq!(eval_expr("'world'", &r, None).unwrap().to_string(), "world");
}

#[test]
fn test_boolean_literals() {
    let r: Vec<String> = vec![];

    assert_eq!(eval_expr("true", &r, None).unwrap().to_string(), "true");
    assert_eq!(eval_expr("false", &r, None).unwrap().to_string(), "false");
}

#[test]
fn test_null_literal() {
    let r: Vec<String> = vec![];

    assert_eq!(eval_expr("null", &r, None).unwrap().to_string(), "null");
}

#[test]
fn test_complex_expression() {
    let h = headers(&["a", "b", "c"]);
    let r = row(&["10", "20", "3"]);

    // (a + b) * c / 2 = (10 + 20) * 3 / 2 = 45
    let result = eval_expr("(@a + @b) * @c / 2", &r, Some(&h)).unwrap();
    assert_eq!(result.to_string(), "45");
}

#[test]
fn test_nested_parentheses() {
    let r = row(&["1", "2", "3", "4"]);

    // ((@1 + @2) * @3) - @4 = ((1 + 2) * 3) - 4 = 5
    assert_eq!(
        eval_expr("((@1 + @2) * @3) - @4", &r, None)
            .unwrap()
            .to_string(),
        "5"
    );
}

#[test]
fn test_whitespace_handling() {
    let r = row(&["10", "20"]);

    // Various whitespace patterns
    assert_eq!(eval_expr("@1+@2", &r, None).unwrap().to_string(), "30");
    assert_eq!(eval_expr("@1 + @2", &r, None).unwrap().to_string(), "30");
    assert_eq!(
        eval_expr("  @1  +  @2  ", &r, None).unwrap().to_string(),
        "30"
    );
}

#[test]
fn test_chained_operations() {
    let r = row(&["2", "3", "4"]);

    // @1 + @2 + @3 = 2 + 3 + 4 = 9
    assert_eq!(
        eval_expr("@1 + @2 + @3", &r, None).unwrap().to_string(),
        "9"
    );

    // @1 * @2 * @3 = 2 * 3 * 4 = 24
    assert_eq!(
        eval_expr("@1 * @2 * @3", &r, None).unwrap().to_string(),
        "24"
    );
}

#[test]
fn test_decimal_numbers() {
    let r = row(&["1.5", "2.5"]);

    assert_eq!(eval_expr("@1 + @2", &r, None).unwrap().to_string(), "4");

    assert_eq!(eval_expr("@1 * @2", &r, None).unwrap().to_string(), "3.75");
}

#[test]
fn test_negative_results() {
    let r = row(&["5", "10"]);

    // 5 - 10 = -5
    assert_eq!(eval_expr("@1 - @2", &r, None).unwrap().to_string(), "-5");
}

#[test]
fn test_large_numbers() {
    let r = row(&["1000000", "1000000"]);

    assert_eq!(
        eval_expr("@1 * @2", &r, None).unwrap().to_string(),
        "1000000000000"
    );
}

#[test]
fn test_comparison_operators() {
    let r = row(&["10", "5"]);

    assert_eq!(eval_expr("@1 > @2", &r, None).unwrap().to_string(), "true");
    assert_eq!(eval_expr("@1 < @2", &r, None).unwrap().to_string(), "false");
    assert_eq!(
        eval_expr("@1 == @2", &r, None).unwrap().to_string(),
        "false"
    );
    assert_eq!(eval_expr("@1 != @2", &r, None).unwrap().to_string(), "true");
    assert_eq!(eval_expr("@1 >= 10", &r, None).unwrap().to_string(), "true");
    assert_eq!(eval_expr("@1 <= 10", &r, None).unwrap().to_string(), "true");
}

#[test]
fn test_string_comparison_operators() {
    let r = row(&["alice", "bob"]);

    // String comparison (lexicographic)
    assert_eq!(eval_expr("@1 eq @1", &r, None).unwrap().to_string(), "true");
    assert_eq!(
        eval_expr("@1 eq @2", &r, None).unwrap().to_string(),
        "false"
    );
    assert_eq!(eval_expr("@1 ne @2", &r, None).unwrap().to_string(), "true");
    assert_eq!(eval_expr("@1 lt @2", &r, None).unwrap().to_string(), "true"); // alice < bob
    assert_eq!(eval_expr("@1 le @2", &r, None).unwrap().to_string(), "true");
    assert_eq!(
        eval_expr("@1 gt @2", &r, None).unwrap().to_string(),
        "false"
    );
    assert_eq!(
        eval_expr("@1 ge @2", &r, None).unwrap().to_string(),
        "false"
    );

    // String literal comparison
    assert_eq!(
        eval_expr("'hello' eq 'hello'", &r, None)
            .unwrap()
            .to_string(),
        "true"
    );
    assert_eq!(
        eval_expr("'abc' lt 'def'", &r, None).unwrap().to_string(),
        "true"
    );
}

#[test]
fn test_logical_operators() {
    let r: Vec<String> = vec![];

    assert_eq!(
        eval_expr("true and true", &r, None).unwrap().to_string(),
        "true"
    );
    assert_eq!(
        eval_expr("true and false", &r, None).unwrap().to_string(),
        "false"
    );
    assert_eq!(
        eval_expr("true or false", &r, None).unwrap().to_string(),
        "true"
    );
    assert_eq!(
        eval_expr("false or false", &r, None).unwrap().to_string(),
        "false"
    );
}

#[test]
fn test_unary_operators() {
    let r: Vec<String> = vec![];

    assert_eq!(eval_expr("-5", &r, None).unwrap().to_string(), "-5");
    assert_eq!(
        eval_expr("not true", &r, None).unwrap().to_string(),
        "false"
    );
    assert_eq!(
        eval_expr("not false", &r, None).unwrap().to_string(),
        "true"
    );
}

#[test]
fn test_power_operator() {
    let r: Vec<String> = vec![];

    assert_eq!(eval_expr("2 ** 3", &r, None).unwrap().to_string(), "8");
    assert_eq!(eval_expr("3 ** 2", &r, None).unwrap().to_string(), "9");
}

#[test]
fn test_modulo_operator() {
    let r: Vec<String> = vec![];

    assert_eq!(eval_expr("10 % 3", &r, None).unwrap().to_string(), "1");
    assert_eq!(eval_expr("15 % 4", &r, None).unwrap().to_string(), "3");
}
