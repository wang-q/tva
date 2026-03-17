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
fn expr_list_expansion_basic() {
    // Test basic list expansion - returns multiple columns
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
    // Should have 2 columns separated by tab
    assert_eq!(lines.len(), 1, "Expected 1 output line, got: {}", stdout);
    let parts: Vec<&str> = lines[0].split('\t').collect();
    assert_eq!(parts.len(), 2, "Expected 2 columns, got: {}", lines[0]);
    assert_eq!(
        parts[0], "326",
        "Expected '326' in first column, got: {}",
        parts[0]
    );
    assert_eq!(
        parts[1], "0.23",
        "Expected '0.23' in second column, got: {}",
        parts[1]
    );
}

#[test]
fn expr_list_expansion_with_header() {
    // Test list expansion with header - header should also expand
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
    // First line should be the expanded header
    let header_parts: Vec<&str> = lines[0].split('\t').collect();
    assert_eq!(
        header_parts.len(),
        2,
        "Expected 2 header columns, got: {}",
        lines[0]
    );
    assert_eq!(
        header_parts[0], "estimate",
        "Expected 'estimate' in first header, got: {}",
        header_parts[0]
    );
    assert_eq!(
        header_parts[1], "variable",
        "Expected 'variable' in second header, got: {}",
        header_parts[1]
    );

    // Data should also have 2 columns
    let data_parts: Vec<&str> = lines[1].split('\t').collect();
    assert_eq!(
        data_parts.len(),
        2,
        "Expected 2 data columns, got: {}",
        lines[1]
    );
}

#[test]
fn expr_list_expansion_with_as_binding() {
    // Test list expansion with 'as' binding for custom headers
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
    assert_eq!(parts.len(), 2, "Expected 2 columns, got: {}", lines[0]);
    assert_eq!(
        parts[0], "326",
        "Expected '326' in first column, got: {}",
        parts[0]
    );
    assert_eq!(
        parts[1], "0.23",
        "Expected '0.23' in second column, got: {}",
        parts[1]
    );
}

#[test]
fn expr_list_expansion_with_expressions() {
    // Test list expansion with computed expressions
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
    assert_eq!(parts.len(), 2, "Expected 2 columns, got: {}", lines[0]);
    assert_eq!(
        parts[0], "200",
        "Expected '200' (100*2) in first column, got: {}",
        parts[0]
    );
    assert_eq!(
        parts[1], "3",
        "Expected '3' (2+1) in second column, got: {}",
        parts[1]
    );
}

#[test]
fn expr_list_expansion_header_with_as() {
    // Test that list expansion with 'as' binding generates correct headers
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
    // Header should use the 'as' binding names
    let header_parts: Vec<&str> = lines[0].split('\t').collect();
    assert_eq!(
        header_parts.len(),
        2,
        "Expected 2 header columns, got: {}",
        lines[0]
    );
    assert_eq!(
        header_parts[0], "e",
        "Expected 'e' in first header (from as @e), got: {}",
        header_parts[0]
    );
    assert_eq!(
        header_parts[1], "v",
        "Expected 'v' in second header (from as @v), got: {}",
        header_parts[1]
    );
}

#[test]
fn expr_multiple_underscore_placeholders() {
    // Multiple _ in same call should all get the piped value
    // "hello" | replace(replace(_, "l", "L"), "o", "O")
    // Should produce "heLLO"
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "'hello' | replace(replace(_, 'l', 'L'), 'o', 'O')",
        ])
        .run();

    assert!(
        stdout.contains("heLLO"),
        "Expected 'heLLO' in stdout, got: {}",
        stdout
    );
}

#[test]
fn expr_single_arg_function_without_underscore() {
    // Single-arg functions can omit _: "hello" | upper()
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "'hello' | upper()"])
        .run();

    assert!(
        stdout.contains("HELLO"),
        "Expected 'HELLO' in stdout, got: {}",
        stdout
    );
}

#[test]
fn expr_single_arg_function_with_underscore() {
    // Single-arg functions can also use _: "hello" | upper(_)
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "'hello' | upper(_)"])
        .run();

    assert!(
        stdout.contains("HELLO"),
        "Expected 'HELLO' in stdout, got: {}",
        stdout
    );
}

#[test]
fn expr_multi_arg_function_without_underscore_errors() {
    // Multi-arg functions without _ should error
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
fn expr_multi_arg_function_with_underscore() {
    // Multi-arg functions with _ should work
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "'hello' | substr(_, 1, 2)"])
        .run();

    assert!(
        stdout.contains("el"),
        "Expected 'el' in stdout, got: {}",
        stdout
    );
}

#[test]
fn expr_split_join_require_underscore() {
    // split and join require 2 args, must use _
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
fn expr_split_without_underscore_errors() {
    // split without _ should error
    let (_, stderr) = TvaCmd::new()
        .args(&["expr", "-E", "'a,b,c' | split(',')"])
        .run();

    assert!(
        stderr.contains("expected 2 arguments") || stderr.contains("got 1"),
        "Expected arity error for split, got: {}",
        stderr
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

#[test]
fn expr_header_format_single_expression() {
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
    // Header should be the formatted expression
    assert_eq!(
        lines[0], "@estimate * 2",
        "Expected header '@estimate * 2', got: {}",
        lines[0]
    );
}

#[test]
fn expr_header_format_last_expression() {
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
    // Header should be the last expression, not the whole expression string
    assert_eq!(
        lines[0], "@e + 100",
        "Expected header '@e + 100' (last expression), got: {}",
        lines[0]
    );
}

#[test]
fn expr_skip_null_with_rows() {
    // Test --skip-null with inline row data
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
    // Should only have 2 lines (Alice and Charlie), Bob's null result should be skipped
    assert_eq!(
        lines.len(),
        2,
        "Expected 2 output lines with --skip-null, got: {}",
        stdout
    );
    assert!(
        lines[0].contains("Alice"),
        "Expected 'Alice' in first line, got: {}",
        lines[0]
    );
    assert!(
        lines[1].contains("Charlie"),
        "Expected 'Charlie' in second line, got: {}",
        lines[1]
    );
}

#[test]
fn expr_skip_null_short_flag() {
    // Test -s short flag for --skip-null
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
    // Should only have 2 lines (Alice and Charlie), Bob's null result should be skipped
    assert_eq!(
        lines.len(),
        2,
        "Expected 2 output lines with -s, got: {}",
        stdout
    );
    assert!(
        lines[0].contains("Alice"),
        "Expected 'Alice' in first line, got: {}",
        lines[0]
    );
    assert!(
        lines[1].contains("Charlie"),
        "Expected 'Charlie' in second line, got: {}",
        lines[1]
    );
}

#[test]
fn expr_without_skip_null_includes_null() {
    // Test that without --skip-null, null results are included
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
    // Should have 3 lines including null
    assert_eq!(
        lines.len(),
        3,
        "Expected 3 output lines without --skip-null, got: {}",
        stdout
    );
    assert!(
        lines[0].contains("Alice"),
        "Expected 'Alice' in first line, got: {}",
        lines[0]
    );
    assert!(
        lines[1].contains("null"),
        "Expected 'null' in second line, got: {}",
        lines[1]
    );
    assert!(
        lines[2].contains("Charlie"),
        "Expected 'Charlie' in third line, got: {}",
        lines[2]
    );
}

#[test]
fn expr_bind_with_pipe() {
    // Test that 'as' binding can be followed by pipe operator
    // [1,2,3] as @list | len() should bind @list and then pipe to len()
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "[1, 2, 3] as @list | len()"])
        .run();

    assert!(
        stdout.contains("3"),
        "Expected '3' (length of list) in stdout, got: {}",
        stdout
    );
}

#[test]
fn expr_bind_with_pipe_chained() {
    // Test chained pipes after bind
    // "hello" as @s | upper() | len() should return 5
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "'hello' as @s | upper() | len()"])
        .run();

    assert!(
        stdout.contains("5"),
        "Expected '5' (length of 'HELLO') in stdout, got: {}",
        stdout
    );
}

#[test]
fn expr_bind_with_pipe_using_bound_var() {
    // Test that bound variable can be used in subsequent expression
    // [1, 2, 3, 4] as @list; @list | len() should work
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "[1, 2, 3, 4] as @list; @list | len()"])
        .run();

    assert!(
        stdout.contains("4"),
        "Expected '4' (length of list) in stdout, got: {}",
        stdout
    );
}

#[test]
fn expr_from_file() {
    // Test -F/--expr-file option
    use std::io::Write;

    // Create a temporary expression file
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
fn expr_from_file_long_flag() {
    // Test --expr-file long flag
    use std::io::Write;

    // Create a temporary expression file
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
fn expr_file_not_found() {
    // Test error handling when expression file doesn't exist
    let (_, stderr) = TvaCmd::new()
        .args(&["expr", "-F", "/nonexistent/file.expr"])
        .run();

    assert!(
        stderr.contains("Failed to read expression file") || stderr.contains("error"),
        "Expected error message for missing file, got: {}",
        stderr
    );
}

#[test]
fn expr_underscore_placeholder_basic() {
    // Test basic underscore usage: "hello" | upper()
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "'hello' | upper(_)"])
        .run();

    assert!(
        stdout.contains("HELLO"),
        "Expected 'HELLO' in stdout, got: {}",
        stdout
    );
}

#[test]
fn expr_underscore_placeholder_with_data() {
    // Test underscore with column data
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
fn expr_underscore_placeholder_nested() {
    // Test nested function with underscore: "hello" | print(substr(_, 1, 2))
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "'hello' | print(substr(_, 1, 2))"])
        .run();

    assert!(
        stdout.contains("el"),
        "Expected 'el' in stdout, got: {}",
        stdout
    );
}

#[test]
fn expr_underscore_placeholder_chained() {
    // Test chained pipes with underscore: "hello" | upper() | substr(_, 1, 3)
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "'hello' | upper() | substr(_, 1, 3)"])
        .run();

    assert!(
        stdout.contains("ELL"),
        "Expected 'ELL' in stdout, got: {}",
        stdout
    );
}

#[test]
fn expr_underscore_placeholder_with_position() {
    // Test underscore in non-first position: "hello" | replace(_, "l", "L")
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "'hello' | replace(_, 'l', 'L')"])
        .run();

    assert!(
        stdout.contains("heLLo"),
        "Expected 'heLLo' in stdout, got: {}",
        stdout
    );
}

#[test]
fn expr_underscore_placeholder_multiple_args() {
    // Test underscore with multiple args: "hello world" | substr(_, 0, 5)
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "'hello world' | substr(_, 0, 5)"])
        .run();

    assert!(
        stdout.contains("hello"),
        "Expected 'hello' in stdout, got: {}",
        stdout
    );
}

#[test]
fn expr_underscore_placeholder_with_file() {
    // Test underscore with real file data
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
fn expr_add_mode_basic() {
    // Test add mode: append expression result to original row
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr", "-n", "name,age", "-r", "Alice,30", "-r", "Bob,25", "-m", "add",
            "-E", "@age * 2",
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // Should have 2 lines (Alice and Bob) with original data + expression result
    assert_eq!(lines.len(), 2, "Expected 2 output lines, got: {}", stdout);
    // Check first line: Alice, 30, 60
    let parts: Vec<&str> = lines[0].split('\t').collect();
    assert_eq!(
        parts.len(),
        3,
        "Expected 3 columns in first line, got: {}",
        lines[0]
    );
    assert_eq!(parts[0], "Alice", "Expected 'Alice' in first column");
    assert_eq!(parts[1], "30", "Expected '30' in second column");
    assert_eq!(parts[2], "60", "Expected '60' in third column");
    // Check second line: Bob, 25, 50
    let parts: Vec<&str> = lines[1].split('\t').collect();
    assert_eq!(
        parts.len(),
        3,
        "Expected 3 columns in second line, got: {}",
        lines[1]
    );
    assert_eq!(parts[2], "50", "Expected '50' in third column");
}

#[test]
fn expr_add_mode_with_header() {
    // Test add mode with header: original headers + expression header
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-H",
            "-m",
            "add",
            "-E",
            "@estimate * 2",
            "tests/data/expr/us_rent_income.tsv",
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // First line should be original headers + expression header
    let header_parts: Vec<&str> = lines[0].split('\t').collect();
    // Original file has 5 columns (GEOID, NAME, variable, estimate, moe)
    // Plus 1 expression column = 6 columns
    assert!(
        header_parts.len() >= 5,
        "Expected at least 5 header columns, got: {}",
        lines[0]
    );
    // Last header should be the expression
    assert_eq!(
        header_parts[header_parts.len() - 1],
        "@estimate * 2",
        "Expected '@estimate * 2' as last header, got: {}",
        header_parts[header_parts.len() - 1]
    );
    // Check data line has same number of columns
    let data_parts: Vec<&str> = lines[1].split('\t').collect();
    assert_eq!(
        data_parts.len(),
        header_parts.len(),
        "Data columns should match header columns"
    );
}

#[test]
fn expr_add_mode_list_expansion() {
    // Test add mode with list expansion: multiple columns appended
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-n",
            "name,age",
            "-r",
            "Alice,30",
            "-m",
            "add",
            "-E",
            "[@age, @age * 2, @age + 10]",
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // Should have 1 line with original 2 columns + 3 expression columns = 5 columns
    let parts: Vec<&str> = lines[0].split('\t').collect();
    assert_eq!(
        parts.len(),
        5,
        "Expected 5 columns (2 original + 3 list items), got: {}",
        lines[0]
    );
    assert_eq!(parts[0], "Alice", "Expected 'Alice' in first column");
    assert_eq!(parts[1], "30", "Expected '30' in second column");
    assert_eq!(parts[2], "30", "Expected '30' (@age) in third column");
    assert_eq!(parts[3], "60", "Expected '60' (@age * 2) in fourth column");
    assert_eq!(parts[4], "40", "Expected '40' (@age + 10) in fifth column");
}

#[test]
fn expr_add_mode_short_flag() {
    // Test -m a short flag for add mode
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
    assert_eq!(
        parts.len(),
        3,
        "Expected 3 columns with -m a, got: {}",
        lines[0]
    );
    assert_eq!(parts[2], "200", "Expected '200' in third column");
}

#[test]
fn expr_add_mode_with_as_binding() {
    // Test add mode with 'as' binding for custom header
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-H",
            "-m",
            "add",
            "-E",
            "@estimate / @moe as @ratio",
            "tests/data/expr/us_rent_income.tsv",
        ])
        .run();

    let lines: Vec<&str> = stdout.lines().collect();
    // First line should be original headers + 'ratio'
    let header_parts: Vec<&str> = lines[0].split('\t').collect();
    assert_eq!(
        header_parts[header_parts.len() - 1],
        "ratio",
        "Expected 'ratio' as last header from 'as @ratio', got: {}",
        header_parts[header_parts.len() - 1]
    );
}

#[test]
fn expr_mutate_mode_basic() {
    // Test mutate mode: modify specified column in place
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
    // Should have 2 lines with modified age column
    assert_eq!(lines.len(), 2, "Expected 2 output lines, got: {}", stdout);
    // Check first line: Alice, 31 (age + 1)
    let parts: Vec<&str> = lines[0].split('\t').collect();
    assert_eq!(
        parts.len(),
        2,
        "Expected 2 columns in first line, got: {}",
        lines[0]
    );
    assert_eq!(parts[0], "Alice", "Expected 'Alice' in first column");
    assert_eq!(parts[1], "31", "Expected '31' (age + 1) in second column");
    // Check second line: Bob, 26
    let parts: Vec<&str> = lines[1].split('\t').collect();
    assert_eq!(parts[1], "26", "Expected '26' (age + 1) in second column");
}

#[test]
fn expr_mutate_mode_with_header() {
    // Test mutate mode preserves original header
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
    // First line should be original headers (not modified)
    let header_parts: Vec<&str> = lines[0].split('\t').collect();
    // Original file has columns: GEOID, NAME, variable, estimate, moe
    assert!(
        header_parts.contains(&"estimate"),
        "Expected 'estimate' in headers, got: {}",
        lines[0]
    );
    // Should not have any new column names
    assert!(
        !header_parts.iter().any(|h| h.contains('*')),
        "Header should not contain expression, got: {}",
        lines[0]
    );
}

#[test]
fn expr_mutate_mode_short_flag() {
    // Test -m u short flag for mutate mode
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
    assert_eq!(
        parts.len(),
        2,
        "Expected 2 columns with -m u, got: {}",
        lines[0]
    );
    // In mutate mode, @price is modified to @price * @qty = 100 * 2 = 200
    assert_eq!(
        parts[0], "200",
        "Expected '200' (price * qty) in first column"
    );
    assert_eq!(
        parts[1], "2",
        "Expected '2' (qty unchanged) in second column"
    );
}

#[test]
fn expr_mutate_mode_requires_as_binding() {
    // Test that mutate mode requires 'as @column' binding
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
fn expr_mutate_mode_column_not_found() {
    // Test error when target column doesn't exist
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
