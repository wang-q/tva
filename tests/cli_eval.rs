#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn eval_simple_arithmetic() {
    let (stdout, _) = TvaCmd::new()
        .args(&["eval", "10 + 20"])
        .run();

    assert!(stdout.contains("30"), "Expected '30' in stdout, got: {}", stdout);
}

#[test]
fn eval_with_headers_and_row() {
    let (stdout, _) = TvaCmd::new()
        .args(&["eval", "-H", "price,qty", "-r", "100,2", "@price * @qty"])
        .run();

    assert!(stdout.contains("200"), "Expected '200' in stdout, got: {}", stdout);
}

#[test]
fn eval_multiple_rows() {
    let (stdout, _) = TvaCmd::new()
        .args(&["eval", "-H", "price,qty", "-r", "100,2", "-r", "200,3", "@price * @qty"])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2, "Expected 2 output lines, got: {}", stdout);
    assert!(lines[0].contains("200"), "Expected '200' in first line, got: {}", lines[0]);
    assert!(lines[1].contains("600"), "Expected '600' in second line, got: {}", lines[1]);
}

#[test]
fn eval_string_function() {
    let (stdout, _) = TvaCmd::new()
        .args(&["eval", "-H", "name", "-r", "  alice  ", "upper(trim(@name))"])
        .run();

    assert!(stdout.contains("ALICE"), "Expected 'ALICE' in stdout, got: {}", stdout);
}

#[test]
fn eval_conditional_expression() {
    let (stdout, _) = TvaCmd::new()
        .args(&["eval", "-H", "score", "-r", "85", "if(@score >= 70, \"pass\", \"fail\")"])
        .run();

    assert!(stdout.contains("pass"), "Expected 'pass' in stdout, got: {}", stdout);
}

#[test]
fn eval_conditional_expression_false() {
    let (stdout, _) = TvaCmd::new()
        .args(&["eval", "-H", "score", "-r", "65", "if(@score >= 70, \"pass\", \"fail\")"])
        .run();

    assert!(stdout.contains("fail"), "Expected 'fail' in stdout, got: {}", stdout);
}

#[test]
fn eval_numeric_functions() {
    let (stdout, _) = TvaCmd::new()
        .args(&["eval", "abs(-5)"])
        .run();

    assert!(stdout.contains("5"), "Expected '5' in stdout, got: {}", stdout);
}

#[test]
fn eval_min_function() {
    let (stdout, _) = TvaCmd::new()
        .args(&["eval", "min(10, 5, 3)"])
        .run();

    assert!(stdout.contains("3"), "Expected '3' in stdout, got: {}", stdout);
}

#[test]
fn eval_power_operator() {
    let (stdout, _) = TvaCmd::new()
        .args(&["eval", "2 ** 10"])
        .run();

    assert!(stdout.contains("1024"), "Expected '1024' in stdout, got: {}", stdout);
}

#[test]
fn eval_modulo_operator() {
    let (stdout, _) = TvaCmd::new()
        .args(&["eval", "10 % 3"])
        .run();

    assert!(stdout.contains("1"), "Expected '1' in stdout, got: {}", stdout);
}

#[test]
fn eval_invalid_expression_error() {
    let (_, stderr) = TvaCmd::new()
        .args(&["eval", "invalid("])
        .run();

    assert!(stderr.contains("Failed to parse expression"), "Expected parse error in stderr, got: {}", stderr);
}

#[test]
fn eval_unknown_function_error() {
    let (_, stderr) = TvaCmd::new()
        .args(&["eval", "unknown(1)"])
        .run();

    assert!(stderr.contains("Unknown function") || stderr.contains("error"), "Expected function error in stderr, got: {}", stderr);
}
