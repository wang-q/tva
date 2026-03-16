#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn expr_global_var_index() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "name",
            "-r",
            "alice",
            "-r",
            "bob",
            "-r",
            "charlie",
            "-E",
            "@__index",
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3, "Expected 3 output lines, got: {}", stdout);
    assert!(lines[0].contains("1"), "Expected '1' in first line, got: {}", lines[0]);
    assert!(lines[1].contains("2"), "Expected '2' in second line, got: {}", lines[1]);
    assert!(lines[2].contains("3"), "Expected '3' in third line, got: {}", lines[2]);
}

#[test]
fn expr_global_var_accumulator() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "value",
            "-r",
            "10",
            "-r",
            "20",
            "-r",
            "30",
            "-E",
            "default(@__sum, 0) + @value as @__sum",
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3, "Expected 3 output lines, got: {}", stdout);
    assert!(lines[0].contains("10"), "Expected '10' in first line, got: {}", lines[0]);
    assert!(lines[1].contains("30"), "Expected '30' in second line, got: {}", lines[1]);
    assert!(lines[2].contains("60"), "Expected '60' in third line, got: {}", lines[2]);
}

#[test]
fn expr_global_var_counter() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "name",
            "-r",
            "a",
            "-r",
            "b",
            "-r",
            "c",
            "-E",
            "default(@__counter, 0) + 1 as @__counter",
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3, "Expected 3 output lines, got: {}", stdout);
    assert!(lines[0].contains("1"), "Expected '1' in first line, got: {}", lines[0]);
    assert!(lines[1].contains("2"), "Expected '2' in second line, got: {}", lines[1]);
    assert!(lines[2].contains("3"), "Expected '3' in third line, got: {}", lines[2]);
}

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
    // First line is header, check data lines contain the filename
    assert!(
        lines[1].contains("us_rent_income.tsv"),
        "Expected filename in output, got: {}",
        lines[1]
    );
}

#[test]
fn expr_global_var_persists_across_rows() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "val",
            "-r",
            "1",
            "-r",
            "2",
            "-r",
            "3",
            "-E",
            "default(@__sum, 0) + @val as @__sum",
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3, "Expected 3 output lines, got: {}", stdout);
    assert!(lines[0].contains("1"), "Expected '1' in first line, got: {}", lines[0]);
    assert!(lines[1].contains("3"), "Expected '3' in second line, got: {}", lines[1]);
    assert!(lines[2].contains("6"), "Expected '6' in third line, got: {}", lines[2]);
}

#[test]
fn expr_global_var_string_concat() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "name",
            "-r",
            "alice",
            "-r",
            "bob",
            "-r",
            "charlie",
            "-E",
            "default(@__names, '') ++ @name ++ ',' as @__names",
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3, "Expected 3 output lines, got: {}", stdout);
    assert!(lines[0].contains("alice,"), "Expected 'alice,' in first line, got: {}", lines[0]);
    assert!(lines[1].contains("alice,bob,"), "Expected 'alice,bob,' in second line, got: {}", lines[1]);
    assert!(lines[2].contains("alice,bob,charlie,"), "Expected 'alice,bob,charlie,' in third line, got: {}", lines[2]);
}

#[test]
fn expr_global_var_read_and_write() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "val",
            "-r",
            "5",
            "-r",
            "3",
            "-r",
            "2",
            "-E",
            "default(@__total, 1) * @val as @__total",
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3, "Expected 3 output lines, got: {}", stdout);
    // First row: 1 * 5 = 5
    // Second row: 5 * 3 = 15
    // Third row: 15 * 2 = 30
    assert!(lines[0].contains("5"), "Expected '5' in first line, got: {}", lines[0]);
    assert!(lines[1].contains("15"), "Expected '15' in second line, got: {}", lines[1]);
    assert!(lines[2].contains("30"), "Expected '30' in third line, got: {}", lines[2]);
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
    // First line is header, data lines start from 1
    assert!(lines[1].contains("1"), "Expected '1' in first data line, got: {}", lines[1]);
    assert!(lines[2].contains("2"), "Expected '2' in second data line, got: {}", lines[2]);
    assert!(lines[3].contains("3"), "Expected '3' in third data line, got: {}", lines[3]);
}

#[test]
fn expr_global_var_null_default() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "val",
            "-r",
            "1",
            "-E",
            "@__unset",
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "Expected 1 output line, got: {}", stdout);
    assert!(lines[0].contains("null"), "Expected 'null' for unset global variable, got: {}", lines[0]);
}
