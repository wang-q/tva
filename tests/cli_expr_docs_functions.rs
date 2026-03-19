#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use test_case::test_case;

// =============================================================================
// Expr Functions Documentation Tests
// =============================================================================
// This file contains tests for expr functions documented in docs/expr/functions.md
//
// Covered function categories:
// - Numeric Operations
// - String Manipulation
// - List Operations
// - Type Conversions
// - Logical Functions
// - Regex Functions
// - Hash Functions
// - DateTime Functions
// - Meta Functions
//
// Last updated: 2026-03-19
// =============================================================================

// =============================================================================
// Numeric Operations (functions.md#L7-L23)
// =============================================================================

// Single-argument numeric functions
#[test_case("abs(-42)", "42" ; "abs")]
#[test_case("ceil(3.14)", "4" ; "ceil")]
#[test_case("floor(3.14)", "3" ; "floor")]
#[test_case("round(3.5)", "4" ; "round")]
#[test_case("sqrt(16)", "4" ; "sqrt")]
#[test_case("ln(1)", "0" ; "ln")]
#[test_case("log10(100)", "2" ; "log10")]
#[test_case("exp(0)", "1" ; "exp")]
fn test_numeric_single_arg(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// Multi-argument numeric functions
#[test_case("pow(2, 10)", "1024" ; "pow")]
#[test_case("max(1, 5, 3, 9, 2)", "9" ; "max")]
#[test_case("min(1, 5, 3, -2, 2)", "-2" ; "min")]
fn test_numeric_multi_arg(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// Type conversion functions
#[test_case("int(\"42\")", "42" ; "int_conversion")]
#[test_case("float(\"3.14\")", "3.14" ; "float_conversion")]
fn test_numeric_type_conversion(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}

// Trigonometric functions
#[test_case("sin(0)", "0" ; "sin")]
#[test_case("cos(0)", "1" ; "cos")]
#[test_case("tan(0)", "0" ; "tan")]
fn test_numeric_trigonometric(expr: &str, expected: &str) {
    let (stdout, _) = TvaCmd::new().args(&["expr", "-E", expr]).run();
    assert!(
        stdout.contains(expected),
        "Expected '{}' for {}, got: {}",
        expected,
        expr,
        stdout
    );
}
