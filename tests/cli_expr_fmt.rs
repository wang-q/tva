#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use test_case::test_case;

// ============================================================================
// Basic Formatting Tests
// ============================================================================

#[test_case("fmt(\"Hello, %()!\", \"world\")", "Hello, world!" ; "basic_hello")]
#[test_case("fmt(\"%() + %() = %()\", 1, 2, 3)", "1 + 2 = 3" ; "multiple_args")]
#[test_case("fmt(\"%(2) %(1)\", \"world\", \"Hello\")", "Hello world" ; "positional_args")]
fn test_fmt_basic(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' in stdout, got: {}",
        expected,
        stdout
    );
}

// ============================================================================
// Format Specifiers Tests
// ============================================================================

#[test_case("fmt(\"%(:>10)\", \"hi\")", "        hi" ; "align_right")]
#[test_case("fmt(\"%(:*<10)\", \"hi\")", "hi********" ; "align_left_fill")]
#[test_case("fmt(\"%(:^10)\", \"hi\")", "    hi    " ; "align_center")]
#[test_case("fmt(\"%(:+)\", 42)", "+42" ; "sign_always")]
#[test_case("fmt(\"%(:08)\", 42)", "00000042" ; "zero_pad")]
#[test_case("fmt(\"%(:.2)\", 3.14159)", "3.14" ; "float_precision")]
#[test_case("fmt(\"%(:b)\", 42)", "101010" ; "binary")]
#[test_case("fmt(\"%(:x)\", 255)", "ff" ; "hex")]
#[test_case("fmt(\"%(:#x)\", 255)", "0xff" ; "hex_with_prefix")]
#[test_case("fmt(\"%(:.5)\", \"hello world\")", "hello" ; "string_truncate")]
fn test_fmt_specifiers(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' in stdout, got: {:?}",
        expected,
        stdout
    );
}

// ============================================================================
// Column References Tests
// ============================================================================

#[test_case(
    &["Alice,100"],
    "fmt(\"%(@1) has %(@2) points\")",
    &["Alice has 100 points"]
    ; "by_index"
)]
#[test_case(
    &["Alice,100"],
    "fmt(\"%(): %(@2) points\", @1)",
    &["Alice: 100 points"]
    ; "mixed"
)]
fn test_fmt_column_ref(rows: &[&str], expr: &str, expected: &[&str]) {
    let mut args = vec!["expr", "-E", expr];
    for row in rows {
        args.push("-r");
        args.push(row);
    }
    let (stdout, _) = TvaCmd::new().args(&args).run();
    for exp in expected {
        assert!(
            stdout.contains(exp),
            "Expected '{}' in stdout, got: {}",
            exp,
            stdout
        );
    }
}

// ============================================================================
// Lambda Variables Tests
// ============================================================================

#[test_case(
    "map([1, 2, 3], x => fmt(\"value: %(x)\"))",
    &["value: 1", "value: 2", "value: 3"]
    ; "basic"
)]
#[test_case(
    "map([1, 2, 3], x => fmt(q(value: %[x])))",
    &["value: 1", "value: 2", "value: 3"]
    ; "with_brackets"
)]
fn test_fmt_lambda(expr: &str, expected: &[&str]) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    for exp in expected {
        assert!(
            stdout.contains(exp),
            "Expected '{}' in stdout, got: {}",
            exp,
            stdout
        );
    }
}

// ============================================================================
// Variable References Tests
// ============================================================================

#[test_case("\"Bob\" as @name; fmt(\"Hello, %(@name)!\")", "Hello, Bob!" ; "basic")]
#[test_case("3.14159 as @pi; fmt(\"Pi = %(@pi:.2)\")", "Pi = 3.14" ; "with_format")]
fn test_fmt_variable(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' in stdout, got: {}",
        expected,
        stdout
    );
}

#[test]
fn test_fmt_variable_multiple_formats() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "42 as @num; fmt(\"Hex: %(@num:#x), Bin: %(@num:b)\")",
        ])
        .run();
    assert!(
        stdout.contains("Hex: 0x2a"),
        "Expected 'Hex: 0x2a' in stdout, got: {}",
        stdout
    );
    assert!(
        stdout.contains("Bin: 101010"),
        "Expected 'Bin: 101010' in stdout, got: {}",
        stdout
    );
}

// ============================================================================
// Multi-row Data with Global Variables Tests
// ============================================================================

#[test_case(
    &["Alice,100", "Bob,200"],
    "fmt(\"Hello, %(@1)! from line %(@__index)\")",
    &["Hello, Alice! from line 1", "Hello, Bob! from line 2"]
    ; "line_index"
)]
#[test_case(
    &["Alice,100", "Bob,200"],
    "default(@__sum, 0) + @2 as @__sum; fmt(\"Hello, %(@1)! sum: %(@__sum)\")",
    &["Hello, Alice! sum: 100", "Hello, Bob! sum: 300"]
    ; "accumulate"
)]
fn test_fmt_global_variable(rows: &[&str], expr: &str, expected: &[&str]) {
    let mut args = vec!["expr"];
    for row in rows {
        args.push("-r");
        args.push(row);
    }
    args.push("-E");
    args.push(expr);
    let (stdout, _) = TvaCmd::new().args(&args).run();
    for exp in expected {
        assert!(
            stdout.contains(exp),
            "Expected '{}' in stdout, got: {}",
            exp,
            stdout
        );
    }
}

// ============================================================================
// Delimiter Selection Tests
// ============================================================================

#[test_case("fmt(q(Result: %[:.2]), 3.14159)", "Result: 3.14" ; "square_brackets")]
#[test_case("fmt(q(%{1:+}), 42)", "+42" ; "curly_braces")]
#[test_case("fmt(q(The \"value\" is %[1]), 42)", "The \"value\" is 42" ; "with_q_string")]
fn test_fmt_delimiter(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' in stdout, got: {:?}",
        expected,
        stdout
    );
}

// ============================================================================
// Escape Sequences Tests
// ============================================================================

#[test]
fn test_fmt_escape_percent() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "fmt(\"100%% complete\")"])
        .run();
    assert!(
        stdout.contains("100% complete"),
        "Expected '100% complete' in stdout, got: {:?}",
        stdout
    );
}
