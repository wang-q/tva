#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use test_case::test_case;

// ============================================================================
// Basic Expression Tests
// ============================================================================

#[test_case("10 + 20", "30" ; "simple_arithmetic")]
#[test_case("abs(-5)", "5" ; "numeric_function")]
#[test_case("min(10, 5, 3)", "3" ; "min_function")]
#[test_case("2 ** 10", "1024" ; "power_operator")]
#[test_case("10 % 3", "1" ; "modulo_operator")]
fn test_expr_basic(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' in stdout, got: {}",
        expected,
        stdout
    );
}

#[test]
fn test_expr_with_colnames_and_row() {
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
fn test_expr_multiple_rows() {
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
fn test_expr_string_function() {
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

#[test_case("85", "pass" ; "conditional_true")]
#[test_case("65", "fail" ; "conditional_false")]
fn test_expr_conditional(score: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "score",
            "-r",
            score,
            "-E",
            "if(@score >= 70, \"pass\", \"fail\")",
        ])
        .run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' in stdout, got: {}",
        expected,
        stdout
    );
}

// ============================================================================
// Real File Tests
// ============================================================================

#[test]
fn test_expr_with_real_file() {
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
fn test_expr_with_real_file_column_index() {
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
fn test_expr_with_real_file_arithmetic() {
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
    assert!(
        lines[1].contains("48952"),
        "Expected '48952' in second line, got: {}",
        lines[1]
    );
}

#[test]
fn test_expr_with_real_file_string_concat() {
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
fn test_expr_with_real_file_conditional() {
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
fn test_expr_with_real_file_function_call() {
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
fn test_expr_with_real_file_pipe_operator() {
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
fn test_expr_with_real_file_variable_binding() {
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
    assert!(
        lines[1].contains("24576"),
        "Expected '24576' in second line, got: {}",
        lines[1]
    );
}

// ============================================================================
// List Expansion Tests
// ============================================================================

#[test]
fn test_expr_list_expansion_basic() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "price,carat",
            "-r",
            "326,0.23",
            "-E",
            "[@price, @carat]",
        ])
        .run();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "Expected 1 output line, got: {}", stdout);
    let parts: Vec<&str> = lines[0].split('\t').collect();
    assert_eq!(parts.len(), 2, "Expected 2 columns, got: {}", lines[0]);
    assert_eq!(parts[0], "326");
    assert_eq!(parts[1], "0.23");
}

#[test]
fn test_expr_list_expansion_with_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-H",
            "-E",
            "[@estimate, @variable]",
            "tests/data/expr/us_rent_income.tsv",
        ])
        .run();
    let lines: Vec<&str> = stdout.lines().collect();
    let header_parts: Vec<&str> = lines[0].split('\t').collect();
    assert_eq!(header_parts.len(), 2);
    assert_eq!(header_parts[0], "estimate");
    assert_eq!(header_parts[1], "variable");
    let data_parts: Vec<&str> = lines[1].split('\t').collect();
    assert_eq!(data_parts.len(), 2);
}

#[test]
fn test_expr_list_expansion_with_as_binding() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "price,carat",
            "-r",
            "326,0.23",
            "-E",
            "[@price as @p, @carat as @c]",
        ])
        .run();
    let lines: Vec<&str> = stdout.lines().collect();
    let parts: Vec<&str> = lines[0].split('\t').collect();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0], "326");
    assert_eq!(parts[1], "0.23");
}

#[test]
fn test_expr_list_expansion_with_expressions() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "price,carat",
            "-r",
            "100,2",
            "-E",
            "[@price * 2, @carat + 1]",
        ])
        .run();
    let lines: Vec<&str> = stdout.lines().collect();
    let parts: Vec<&str> = lines[0].split('\t').collect();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0], "200");
    assert_eq!(parts[1], "3");
}

#[test]
fn test_expr_list_expansion_header_with_as() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-H",
            "-E",
            "[@estimate as @e, @variable as @v]",
            "tests/data/expr/us_rent_income.tsv",
        ])
        .run();
    let lines: Vec<&str> = stdout.lines().collect();
    let header_parts: Vec<&str> = lines[0].split('\t').collect();
    assert_eq!(header_parts[0], "e");
    assert_eq!(header_parts[1], "v");
}

// ============================================================================
// Underscore Placeholder Tests
// ============================================================================

#[test_case("'hello' | upper()", "HELLO" ; "single_arg_without_underscore")]
#[test_case("'hello' | upper(_)", "HELLO" ; "single_arg_with_underscore")]
#[test_case("'hello' | replace(replace(_, 'l', 'L'), 'o', 'O')", "heLLO" ; "multiple_underscores")]
#[test_case("'hello' | substr(_, 1, 2)", "el" ; "multi_arg_with_underscore")]
#[test_case("'hello' | print(substr(_, 1, 2))", "el" ; "nested_with_underscore")]
#[test_case("'hello' | upper() | substr(_, 1, 3)", "ELL" ; "chained_with_underscore")]
#[test_case("'hello' | replace(_, 'l', 'L')", "heLLo" ; "underscore_in_position")]
#[test_case("'hello world' | substr(_, 0, 5)", "hello" ; "underscore_multiple_args")]
fn test_expr_underscore_placeholder(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' in stdout, got: {}",
        expected,
        stdout
    );
}

#[test]
fn test_expr_underscore_placeholder_with_data() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "name",
            "-r",
            "alice",
            "-E",
            "@name | upper(_)",
        ])
        .run();
    assert!(
        stdout.contains("ALICE"),
        "Expected 'ALICE' in stdout, got: {}",
        stdout
    );
}

#[test]
fn test_expr_underscore_placeholder_with_file() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-H",
            "-E",
            "@NAME | lower(_)",
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
fn test_expr_split_join_require_underscore() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "'a,b,c' | split(_, ',') | join(_, '-')"])
        .run();
    assert!(
        stdout.contains("a-b-c"),
        "Expected 'a-b-c' in stdout, got: {}",
        stdout
    );
}

#[test]
fn test_expr_multi_arg_function_without_underscore_errors() {
    let (_, stderr) = TvaCmd::new()
        .args(&["expr", "-E", "'hello' | substr(1, 2)"])
        .run();
    assert!(
        stderr.contains("expected 3 arguments") || stderr.contains("got 2"),
        "Expected arity error in stderr, got: {}",
        stderr
    );
}

#[test]
fn test_expr_split_without_underscore_errors() {
    let (_, stderr) = TvaCmd::new()
        .args(&["expr", "-E", "'a,b,c' | split(',')"])
        .run();
    assert!(
        stderr.contains("expected 2 arguments") || stderr.contains("got 1"),
        "Expected arity error for split, got: {}",
        stderr
    );
}

// ============================================================================
// Skip Null Tests
// ============================================================================

#[test]
fn test_expr_skip_null_with_rows() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "score,name",
            "-r",
            "85,Alice",
            "-r",
            "65,Bob",
            "-r",
            "90,Charlie",
            "-E",
            "if(@score >= 70, @name, null)",
            "--mode",
            "skip-null",
        ])
        .run();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("Alice"));
    assert!(lines[1].contains("Charlie"));
}

#[test]
fn test_expr_skip_null_short_flag() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "score,name",
            "-r",
            "85,Alice",
            "-r",
            "65,Bob",
            "-r",
            "90,Charlie",
            "-m",
            "skip-null",
            "-E",
            "if(@score >= 70, @name, null)",
        ])
        .run();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("Alice"));
    assert!(lines[1].contains("Charlie"));
}

#[test]
fn test_expr_without_skip_null_includes_null() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "score,name",
            "-r",
            "85,Alice",
            "-r",
            "65,Bob",
            "-r",
            "90,Charlie",
            "-E",
            "if(@score >= 70, @name, null)",
        ])
        .run();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].contains("Alice"));
    assert!(lines[1].contains("null"));
    assert!(lines[2].contains("Charlie"));
}

// ============================================================================
// Variable Binding Tests
// ============================================================================

#[test]
fn test_expr_bind_with_pipe() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "[1, 2, 3] as @list | len()"])
        .run();
    assert!(
        stdout.contains("3"),
        "Expected '3' in stdout, got: {}",
        stdout
    );
}

#[test]
fn test_expr_bind_with_pipe_chained() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "'hello' as @s | upper() | len()"])
        .run();
    assert!(
        stdout.contains("5"),
        "Expected '5' in stdout, got: {}",
        stdout
    );
}

#[test]
fn test_expr_bind_with_pipe_using_bound_var() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "[1, 2, 3, 4] as @list; @list | len()"])
        .run();
    assert!(
        stdout.contains("4"),
        "Expected '4' in stdout, got: {}",
        stdout
    );
}

// ============================================================================
// Header Format Tests
// ============================================================================

#[test]
fn test_expr_header_format_single_expression() {
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
    assert_eq!(lines[0], "@estimate * 2");
}

#[test]
fn test_expr_header_format_last_expression() {
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
    assert_eq!(lines[0], "@e + 100");
}

// ============================================================================
// Expression File Tests
// ============================================================================

#[test]
fn test_expr_from_file() {
    use std::io::Write;
    let mut expr_file = tempfile::NamedTempFile::new().unwrap();
    writeln!(expr_file, "@price * @qty").unwrap();
    let expr_path = expr_file.path().to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-n", "price,qty", "-r", "100,2", "-F", expr_path])
        .run();
    assert!(
        stdout.contains("200"),
        "Expected '200' in stdout when using -F, got: {}",
        stdout
    );
}

#[test]
fn test_expr_from_file_long_flag() {
    use std::io::Write;
    let mut expr_file = tempfile::NamedTempFile::new().unwrap();
    writeln!(expr_file, "@name | upper()").unwrap();
    let expr_path = expr_file.path().to_str().unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "name",
            "-r",
            "alice",
            "--expr-file",
            expr_path,
        ])
        .run();
    assert!(
        stdout.contains("ALICE"),
        "Expected 'ALICE' in stdout when using --expr-file, got: {}",
        stdout
    );
}

#[test]
fn test_expr_file_not_found() {
    let (_, stderr) = TvaCmd::new()
        .args(&["expr", "-F", "/nonexistent/file.expr"])
        .run();
    assert!(
        stderr.contains("Failed to read expression file") || stderr.contains("error"),
        "Expected error message for missing file, got: {}",
        stderr
    );
}

// ============================================================================
// Extend Mode Tests
// ============================================================================

#[test]
fn test_expr_extend_mode_basic() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr", "-n", "name,age", "-r", "Alice,30", "-r", "Bob,25", "-m", "extend",
            "-E", "@age * 2",
        ])
        .run();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    let parts: Vec<&str> = lines[0].split('\t').collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0], "Alice");
    assert_eq!(parts[1], "30");
    assert_eq!(parts[2], "60");
    let parts: Vec<&str> = lines[1].split('\t').collect();
    assert_eq!(parts[2], "50");
}

#[test]
fn test_expr_extend_mode_with_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-H",
            "-m",
            "extend",
            "-E",
            "@estimate * 2",
            "tests/data/expr/us_rent_income.tsv",
        ])
        .run();
    let lines: Vec<&str> = stdout.lines().collect();
    let header_parts: Vec<&str> = lines[0].split('\t').collect();
    assert!(header_parts.len() >= 5);
    assert_eq!(header_parts[header_parts.len() - 1], "@estimate * 2");
}

#[test]
fn test_expr_extend_mode_list_expansion() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "name,age",
            "-r",
            "Alice,30",
            "-m",
            "extend",
            "-E",
            "[@age, @age * 2, @age + 10]",
        ])
        .run();
    let lines: Vec<&str> = stdout.lines().collect();
    let parts: Vec<&str> = lines[0].split('\t').collect();
    assert_eq!(parts.len(), 5);
    assert_eq!(parts[0], "Alice");
    assert_eq!(parts[1], "30");
    assert_eq!(parts[2], "30");
    assert_eq!(parts[3], "60");
    assert_eq!(parts[4], "40");
}

#[test]
fn test_expr_extend_mode_short_flag() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "price,qty",
            "-r",
            "100,2",
            "-m",
            "a",
            "-E",
            "@price * @qty",
        ])
        .run();
    let lines: Vec<&str> = stdout.lines().collect();
    let parts: Vec<&str> = lines[0].split('\t').collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[2], "200");
}

#[test]
fn test_expr_extend_mode_with_as_binding() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-H",
            "-m",
            "extend",
            "-E",
            "@estimate / @moe as @ratio",
            "tests/data/expr/us_rent_income.tsv",
        ])
        .run();
    let lines: Vec<&str> = stdout.lines().collect();
    let header_parts: Vec<&str> = lines[0].split('\t').collect();
    assert_eq!(header_parts[header_parts.len() - 1], "ratio");
}

// ============================================================================
// Mutate Mode Tests
// ============================================================================

#[test]
fn test_expr_mutate_mode_basic() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "name,age",
            "-r",
            "Alice,30",
            "-r",
            "Bob,25",
            "-m",
            "mutate",
            "-E",
            "@age + 1 as @age",
        ])
        .run();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    let parts: Vec<&str> = lines[0].split('\t').collect();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0], "Alice");
    assert_eq!(parts[1], "31");
    let parts: Vec<&str> = lines[1].split('\t').collect();
    assert_eq!(parts[1], "26");
}

#[test]
fn test_expr_mutate_mode_with_header() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-H",
            "-m",
            "mutate",
            "-E",
            "@estimate * 2 as @estimate",
            "tests/data/expr/us_rent_income.tsv",
        ])
        .run();
    let lines: Vec<&str> = stdout.lines().collect();
    let header_parts: Vec<&str> = lines[0].split('\t').collect();
    assert!(header_parts.contains(&"estimate"));
    assert!(!header_parts.iter().any(|h| h.contains('*')));
}

#[test]
fn test_expr_mutate_mode_short_flag() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "price,qty",
            "-r",
            "100,2",
            "-m",
            "u",
            "-E",
            "@price * @qty as @price",
        ])
        .run();
    let lines: Vec<&str> = stdout.lines().collect();
    let parts: Vec<&str> = lines[0].split('\t').collect();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0], "200");
    assert_eq!(parts[1], "2");
}

#[test]
fn test_expr_mutate_mode_requires_as_binding() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "expr", "-n", "name,age", "-r", "Alice,30", "-m", "mutate", "-E", "@age + 1",
        ])
        .run();
    assert!(
        stderr.contains("mutate mode requires 'as @column' binding"),
        "Expected error about missing 'as @column' binding, got: {}",
        stderr
    );
}

#[test]
fn test_expr_mutate_mode_column_not_found() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "name,age",
            "-r",
            "Alice,30",
            "-m",
            "mutate",
            "-E",
            "@age + 1 as @nonexistent",
        ])
        .run();
    assert!(
        stderr.contains("mutate target column 'nonexistent' not found"),
        "Expected error about column not found, got: {}",
        stderr
    );
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_expr_invalid_expression_error() {
    let (_, stderr) = TvaCmd::new().args(&["expr", "-E", "invalid("]).run();
    assert!(
        stderr.contains("Failed to parse expression"),
        "Expected parse error in stderr, got: {}",
        stderr
    );
}

#[test]
fn test_expr_unknown_function_error() {
    let (_, stderr) = TvaCmd::new().args(&["expr", "-E", "unknown(1)"]).run();
    assert!(
        stderr.contains("Unknown function") || stderr.contains("error"),
        "Expected function error in stderr, got: {}",
        stderr
    );
}
