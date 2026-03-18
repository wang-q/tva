#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

// ============================================================================
// Basic Formatting Tests
// ============================================================================

#[test]
fn fmt_basic_hello_world() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "fmt(\"Hello, %()!\", \"world\")"])
        .run();
    assert!(
        stdout.contains("Hello, world!"),
        "Expected 'Hello, world!' in stdout, got: {}",
        stdout
    );
}

#[test]
fn fmt_multiple_args() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "fmt(\"%() + %() = %()\", 1, 2, 3)"])
        .run();
    assert!(
        stdout.contains("1 + 2 = 3"),
        "Expected '1 + 2 = 3' in stdout, got: {}",
        stdout
    );
}

#[test]
fn fmt_position_args() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "fmt(\"%(2) %(1)\", \"world\", \"Hello\")"])
        .run();
    assert!(
        stdout.contains("Hello world"),
        "Expected 'Hello world' in stdout, got: {}",
        stdout
    );
}

// ============================================================================
// Format Specifiers Tests
// ============================================================================

#[test]
fn fmt_align_right() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "fmt(\"%(:>10)\", \"hi\")"])
        .run();
    assert!(
        stdout.contains("        hi"),
        "Expected right-aligned 'hi' in stdout, got: {:?}",
        stdout
    );
}

#[test]
fn fmt_align_left_with_fill() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "fmt(\"%(:*<10)\", \"hi\")"])
        .run();
    assert!(
        stdout.contains("hi********"),
        "Expected left-aligned with * fill in stdout, got: {:?}",
        stdout
    );
}

#[test]
fn fmt_align_center() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "fmt(\"%(:^10)\", \"hi\")"])
        .run();
    assert!(
        stdout.contains("    hi    "),
        "Expected centered 'hi' in stdout, got: {:?}",
        stdout
    );
}

#[test]
fn fmt_sign_always() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "fmt(\"%(:+)\", 42)"])
        .run();
    assert!(
        stdout.contains("+42"),
        "Expected '+42' in stdout, got: {}",
        stdout
    );
}

#[test]
fn fmt_zero_pad() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "fmt(\"%(:08)\", 42)"])
        .run();
    assert!(
        stdout.contains("00000042"),
        "Expected '00000042' in stdout, got: {}",
        stdout
    );
}

#[test]
fn fmt_float_precision() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "fmt(\"%(:.2)\", 3.14159)"])
        .run();
    assert!(
        stdout.contains("3.14"),
        "Expected '3.14' in stdout, got: {}",
        stdout
    );
}

#[test]
fn fmt_binary() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "fmt(\"%(:b)\", 42)"])
        .run();
    assert!(
        stdout.contains("101010"),
        "Expected '101010' in stdout, got: {}",
        stdout
    );
}

#[test]
fn fmt_hex() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "fmt(\"%(:x)\", 255)"])
        .run();
    assert!(
        stdout.contains("ff"),
        "Expected 'ff' in stdout, got: {}",
        stdout
    );
}

#[test]
fn fmt_hex_with_prefix() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "fmt(\"%(:#x)\", 255)"])
        .run();
    assert!(
        stdout.contains("0xff"),
        "Expected '0xff' in stdout, got: {}",
        stdout
    );
}

#[test]
fn fmt_string_truncate() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "fmt(\"%(:.5)\", \"hello world\")"])
        .run();
    assert!(
        stdout.contains("hello"),
        "Expected 'hello' in stdout, got: {}",
        stdout
    );
}

// ============================================================================
// Column References Tests
// ============================================================================

#[test]
fn fmt_column_ref_by_index() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "fmt(\"%(@1) has %(@2) points\")",
            "-r",
            "Alice,100",
        ])
        .run();
    assert!(
        stdout.contains("Alice has 100 points"),
        "Expected 'Alice has 100 points' in stdout, got: {}",
        stdout
    );
}

#[test]
fn fmt_column_ref_mixed() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-E",
            "fmt(\"%(): %(@2) points\", @1)",
            "-r",
            "Alice,100",
        ])
        .run();
    assert!(
        stdout.contains("Alice: 100 points"),
        "Expected 'Alice: 100 points' in stdout, got: {}",
        stdout
    );
}

// ============================================================================
// Lambda Variables Tests
// ============================================================================

#[test]
fn fmt_lambda_variable() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "map([1, 2, 3], x => fmt(\"value: %(x)\"))"])
        .run();
    assert!(
        stdout.contains("value: 1"),
        "Expected 'value: 1' in stdout, got: {}",
        stdout
    );
    assert!(
        stdout.contains("value: 2"),
        "Expected 'value: 2' in stdout, got: {}",
        stdout
    );
    assert!(
        stdout.contains("value: 3"),
        "Expected 'value: 3' in stdout, got: {}",
        stdout
    );
}

#[test]
fn fmt_lambda_variable_with_brackets() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "map([1, 2, 3], x => fmt(q(value: %[x])))"])
        .run();
    assert!(
        stdout.contains("value: 1"),
        "Expected 'value: 1' in stdout, got: {}",
        stdout
    );
    assert!(
        stdout.contains("value: 2"),
        "Expected 'value: 2' in stdout, got: {}",
        stdout
    );
    assert!(
        stdout.contains("value: 3"),
        "Expected 'value: 3' in stdout, got: {}",
        stdout
    );
}

// ============================================================================
// Variable References Tests
// ============================================================================

#[test]
fn fmt_variable_basic() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "\"Bob\" as @name; fmt(\"Hello, %(@name)!\")"])
        .run();
    assert!(
        stdout.contains("Hello, Bob!"),
        "Expected 'Hello, Bob!' in stdout, got: {}",
        stdout
    );
}

#[test]
fn fmt_variable_with_format() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "3.14159 as @pi; fmt(\"Pi = %(@pi:.2)\")"])
        .run();
    assert!(
        stdout.contains("Pi = 3.14"),
        "Expected 'Pi = 3.14' in stdout, got: {}",
        stdout
    );
}

#[test]
fn fmt_variable_multiple_formats() {
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

#[test]
fn fmt_global_variable_line_index() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-r",
            "Alice,100",
            "-r",
            "Bob,200",
            "-E",
            "fmt(\"Hello, %(@1)! from line %(@__index)\")",
        ])
        .run();
    assert!(
        stdout.contains("Hello, Alice! from line 1"),
        "Expected line 1 output in stdout, got: {}",
        stdout
    );
    assert!(
        stdout.contains("Hello, Bob! from line 2"),
        "Expected line 2 output in stdout, got: {}",
        stdout
    );
}

#[test]
fn fmt_global_variable_accumulate() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "expr",
            "-r",
            "Alice,100",
            "-r",
            "Bob,200",
            "-E",
            "default(@__sum, 0) + @2 as @__sum; fmt(\"Hello, %(@1)! sum: %(@__sum)\")",
        ])
        .run();
    assert!(
        stdout.contains("Hello, Alice! sum: 100"),
        "Expected Alice sum output in stdout, got: {}",
        stdout
    );
    assert!(
        stdout.contains("Hello, Bob! sum: 300"),
        "Expected Bob sum output in stdout, got: {}",
        stdout
    );
}

// ============================================================================
// Delimiter Selection Tests
// ============================================================================

#[test]
fn fmt_delimiter_square_brackets() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "fmt(q(Result: %[:.2]), 3.14159)"])
        .run();
    assert!(
        stdout.contains("Result: 3.14"),
        "Expected 'Result: 3.14' in stdout, got: {}",
        stdout
    );
}

#[test]
fn fmt_delimiter_curly_braces() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "fmt(q(%{1:+}), 42)"])
        .run();
    assert!(
        stdout.contains("+42"),
        "Expected '+42' in stdout, got: {}",
        stdout
    );
}

#[test]
fn fmt_delimiter_with_q_string() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "fmt(q(The \"value\" is %[1]), 42)"])
        .run();
    assert!(
        stdout.contains("The \"value\" is 42"),
        "Expected 'The \"value\" is 42' in stdout, got: {:?}",
        stdout
    );
}

// ============================================================================
// Escape Sequences Tests
// ============================================================================

#[test]
fn fmt_escape_percent() {
    let (stdout, _) = TvaCmd::new()
        .args(&["expr", "-E", "fmt(\"100%% complete\")"])
        .run();
    assert!(
        stdout.contains("100% complete"),
        "Expected '100% complete' in stdout, got: {:?}",
        stdout
    );
}
