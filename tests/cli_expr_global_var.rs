#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use test_case::test_case;

// ============================================================================
// Global Variable Accumulation Tests
// ============================================================================

#[test_case(
    &["10", "20", "30"],
    "default(@__sum, 0) + @value as @__sum",
    &["10", "30", "60"]
    ; "accumulator_sum"
)]
#[test_case(
    &["5", "3", "2"],
    "default(@__total, 1) * @value as @__total",
    &["5", "15", "30"]
    ; "accumulator_product"
)]
fn expr_global_var_accumulation(rows: &[&str], expr: &str, expected: &[&str]) {
    let mut args = vec!["expr", "-n", "value"];
    for row in rows {
        args.push("-r");
        args.push(row);
    }
    args.push("-E");
    args.push(expr);

    let (stdout, _) = TvaCmd::new().args(&args).run();
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(
        lines.len(),
        rows.len(),
        "Expected {} output lines, got: {}",
        rows.len(),
        stdout
    );
    for (i, (line, exp)) in lines.iter().zip(expected.iter()).enumerate() {
        assert!(
            line.contains(exp),
            "Expected '{}' in line {}, got: {}",
            exp,
            i + 1,
            line
        );
    }
}

// ============================================================================
// Global Variable Counter Tests
// ============================================================================

#[test_case(
    &["a", "b", "c"],
    "default(@__counter, 0) + 1 as @__counter",
    &["1", "2", "3"]
    ; "counter_increment"
)]
#[test_case(
    &["alice", "bob", "charlie"],
    "default(@__names, '') ++ @name ++ ',' as @__names",
    &["alice,", "alice,bob,", "alice,bob,charlie,"]
    ; "string_concat"
)]
fn expr_global_var_counter(rows: &[&str], expr: &str, expected: &[&str]) {
    let mut args = vec!["expr", "-n", "name"];
    for row in rows {
        args.push("-r");
        args.push(row);
    }
    args.push("-E");
    args.push(expr);

    let (stdout, _) = TvaCmd::new().args(&args).run();
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(
        lines.len(),
        rows.len(),
        "Expected {} output lines, got: {}",
        rows.len(),
        stdout
    );
    for (i, (line, exp)) in lines.iter().zip(expected.iter()).enumerate() {
        assert!(
            line.contains(exp),
            "Expected '{}' in line {}, got: {}",
            exp,
            i + 1,
            line
        );
    }
}

// ============================================================================
// Index Variable Tests
// ============================================================================

#[test_case(
    &["alice", "bob", "charlie"],
    &["1", "2", "3"]
    ; "index_with_rows"
)]
fn expr_global_var_index(rows: &[&str], expected: &[&str]) {
    let mut args = vec!["expr", "-n", "name"];
    for row in rows {
        args.push("-r");
        args.push(row);
    }
    args.extend_from_slice(&["-E", "@__index"]);

    let (stdout, _) = TvaCmd::new().args(&args).run();
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(
        lines.len(),
        rows.len(),
        "Expected {} output lines, got: {}",
        rows.len(),
        stdout
    );
    for (i, (line, exp)) in lines.iter().zip(expected.iter()).enumerate() {
        assert!(
            line.contains(exp),
            "Expected '{}' in line {}, got: {}",
            exp,
            i + 1,
            line
        );
    }
}

#[test]
fn expr_global_var_index_with_file() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-H",
            "-E",
            "@__index",
            "tests/data/expr/us_rent_income.tsv",
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert!(
        lines[1].contains("1"),
        "Expected '1' in first data line, got: {}",
        lines[1]
    );
    assert!(
        lines[2].contains("2"),
        "Expected '2' in second data line, got: {}",
        lines[2]
    );
    assert!(
        lines[3].contains("3"),
        "Expected '3' in third data line, got: {}",
        lines[3]
    );
}

// ============================================================================
// File Variable Tests
// ============================================================================

#[test]
fn expr_global_var_file() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-H",
            "-E",
            "@__file",
            "tests/data/expr/us_rent_income.tsv",
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert!(
        lines[1].contains("us_rent_income.tsv"),
        "Expected filename in output, got: {}",
        lines[1]
    );
}

// ============================================================================
// Null/Default Tests
// ============================================================================

#[test]
fn expr_global_var_null_default() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-n", "val", "-r", "1", "-E", "@__unset"])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "Expected 1 output line, got: {}", stdout);
    assert!(
        lines[0].contains("null"),
        "Expected 'null' for unset global variable, got: {}",
        lines[0]
    );
}
