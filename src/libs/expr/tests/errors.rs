//! Error handling tests for expression engine

use crate::libs::expr::{eval_expr, parser};

// Helper to create a test row
fn row(values: &[&str]) -> Vec<String> {
    values.iter().map(|s| s.to_string()).collect()
}

// Helper to create headers
fn headers(names: &[&str]) -> Vec<String> {
    names.iter().map(|s| s.to_string()).collect()
}

#[test]
fn test_parse_empty_column_ref() {
    assert!(parser::parse("@").is_err());
}

#[test]
fn test_parse_invalid_column_index_zero() {
    assert!(parser::parse("@0").is_err());
}

#[test]
fn test_parse_unexpected_character() {
    assert!(parser::parse("@1 + $").is_err());
}

#[test]
fn test_parse_invalid_number() {
    assert!(parser::parse("1.2.3").is_err());
}

#[test]
fn test_parse_unclosed_parenthesis() {
    assert!(parser::parse("(@1 + @2").is_err());
}

#[test]
fn test_eval_column_out_of_bounds() {
    let r = row(&["10"]);
    assert!(eval_expr("@2", &r, None).is_err());
}

#[test]
fn test_eval_unknown_column_name() {
    let r = row(&["10"]);
    let h = headers(&["a"]);
    assert!(eval_expr("@unknown", &r, Some(&h)).is_err());
}

#[test]
fn test_eval_division_by_zero() {
    let r = row(&["10", "0"]);
    let result = eval_expr("@1 / @2", &r, None);
    assert!(result.is_err());
}
