#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn expr_simple_arithmetic() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "10 + 20"]).run();

    assert!(
        stdout.contains("30"),
        "Expected '30' in stdout, got: {}",
        stdout
    );
}

#[test]
fn expr_with_colnames_and_row() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "price,qty",
            "-r",
            "100,2",
            "-E",
            "@price * @qty",
        ])
        .run();

    assert!(
        stdout.contains("200"),
        "Expected '200' in stdout, got: {}",
        stdout
    );
}

#[test]
fn expr_multiple_rows() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "price,qty",
            "-r",
            "100,2",
            "-r",
            "200,3",
            "-E",
            "@price * @qty",
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2, "Expected 2 output lines, got: {}", stdout);
    assert!(
        lines[0].contains("200"),
        "Expected '200' in first line, got: {}",
        lines[0]
    );
    assert!(
        lines[1].contains("600"),
        "Expected '600' in second line, got: {}",
        lines[1]
    );
}

#[test]
fn expr_string_function() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "name",
            "-r",
            "  alice  ",
            "-E",
            "upper(trim(@name))",
        ])
        .run();

    assert!(
        stdout.contains("ALICE"),
        "Expected 'ALICE' in stdout, got: {}",
        stdout
    );
}

#[test]
fn expr_conditional_expression() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "score",
            "-r",
            "85",
            "-E",
            "if(@score >= 70, \"pass\", \"fail\")",
        ])
        .run();

    assert!(
        stdout.contains("pass"),
        "Expected 'pass' in stdout, got: {}",
        stdout
    );
}

#[test]
fn expr_conditional_expression_false() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "score",
            "-r",
            "65",
            "-E",
            "if(@score >= 70, \"pass\", \"fail\")",
        ])
        .run();

    assert!(
        stdout.contains("fail"),
        "Expected 'fail' in stdout, got: {}",
        stdout
    );
}

#[test]
fn expr_numeric_functions() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "abs(-5)"]).run();

    assert!(
        stdout.contains("5"),
        "Expected '5' in stdout, got: {}",
        stdout
    );
}

#[test]
fn expr_min_function() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "min(10, 5, 3)"]).run();

    assert!(
        stdout.contains("3"),
        "Expected '3' in stdout, got: {}",
        stdout
    );
}

#[test]
fn expr_power_operator() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "2 ** 10"]).run();

    assert!(
        stdout.contains("1024"),
        "Expected '1024' in stdout, got: {}",
        stdout
    );
}

#[test]
fn expr_modulo_operator() {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", "10 % 3"]).run();

    assert!(
        stdout.contains("1"),
        "Expected '1' in stdout, got: {}",
        stdout
    );
}

#[test]
fn expr_invalid_expression_error() {
    let (_, stderr) = TvaCmd::new().args(&["expr", "-E", "invalid("]).run();

    assert!(
        stderr.contains("Failed to parse expression"),
        "Expected parse error in stderr, got: {}",
        stderr
    );
}

#[test]
fn expr_unknown_function_error() {
    let (_, stderr) = TvaCmd::new().args(&["expr", "-E", "unknown(1)"]).run();

    assert!(
        stderr.contains("Unknown function") || stderr.contains("error"),
        "Expected function error in stderr, got: {}",
        stderr
    );
}

#[test]
fn expr_with_real_file() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-H",
            "-E",
            "@estimate",
            "tests/data/expr/us_rent_income.tsv",
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // First line is the expression itself as header, data starts from line 1
    assert!(
        lines[1].contains("24476"),
        "Expected '24476' in second line, got: {}",
        lines[1]
    );
    assert!(
        lines[2].contains("747"),
        "Expected '747' in third line, got: {}",
        lines[2]
    );
}

#[test]
fn expr_with_real_file_column_index() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-H",
            "-E",
            "@2",
            "tests/data/expr/us_rent_income.tsv",
        ])
        .run();

    assert!(
        stdout.contains("Alabama"),
        "Expected 'Alabama' in output, got: {}",
        stdout
    );
    assert!(
        stdout.contains("Alaska"),
        "Expected 'Alaska' in output, got: {}",
        stdout
    );
}

#[test]
fn expr_with_real_file_arithmetic() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-H",
            "-E",
            "@estimate * 2",
            "tests/data/expr/us_rent_income.tsv",
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // First line is the expression itself as header, data starts from line 1
    assert!(
        lines[1].contains("48952"),
        "Expected '48952' (24476*2) in second line, got: {}",
        lines[1]
    );
}

#[test]
fn expr_with_real_file_string_concat() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-H",
            "-E",
            "@NAME ++ \": \" ++ @variable",
            "tests/data/expr/us_rent_income.tsv",
        ])
        .run();

    assert!(
        stdout.contains("Alabama: income"),
        "Expected 'Alabama: income' in output, got: {}",
        stdout
    );
}

#[test]
fn expr_with_real_file_conditional() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-H",
            "-E",
            "if(@estimate > 1000, \"high\", \"low\")",
            "tests/data/expr/us_rent_income.tsv",
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert!(
        lines[0].contains("high"),
        "Expected 'high' for income 24476, got: {}",
        lines[0]
    );
}

#[test]
fn expr_with_real_file_function_call() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-H",
            "-E",
            "upper(@NAME)",
            "tests/data/expr/us_rent_income.tsv",
        ])
        .run();

    assert!(
        stdout.contains("ALABAMA"),
        "Expected 'ALABAMA' in output, got: {}",
        stdout
    );
}

#[test]
fn expr_with_real_file_pipe_operator() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-H",
            "-E",
            "@NAME | lower()",
            "tests/data/expr/us_rent_income.tsv",
        ])
        .run();

    assert!(
        stdout.contains("alabama"),
        "Expected 'alabama' in output, got: {}",
        stdout
    );
}

#[test]
fn expr_with_real_file_variable_binding() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-H",
            "-E",
            "@estimate as @e; @e + 100",
            "tests/data/expr/us_rent_income.tsv",
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // First line is the expression itself as header, data starts from line 1
    assert!(
        lines[1].contains("24576"),
        "Expected '24576' (24476+100) in second line, got: {}",
        lines[1]
    );
}
