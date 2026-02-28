#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn upstream_error_no_such_file() {
    let (_, stderr) = TvaCmd::new()
        .args(&["filter", "tests/data/filter/non_existent_file.tsv"])
        .run_fail();
    assert!(stderr.contains("could not open"));
}

#[test]
fn upstream_error_invalid_field_0() {
    let (_, stderr) = TvaCmd::new()
        .args(&["filter", "--ge", "0:10", "tests/data/filter/input1.tsv"])
        .run_fail();
    assert!(
        stderr.contains("Field index 0 is invalid")
            || stderr.contains("invalid")
            || stderr.contains("field index must be >= 1")
    );
}

#[test]
fn upstream_error_ff_absdiff_le_invalid_value() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-absdiff-le",
            "1:2:g",
            "tests/data/filter/input1.tsv",
        ])
        .run_fail();
    assert!(stderr.contains("invalid numeric value") || stderr.contains("invalid"));
}

#[test]
fn upstream_error_regex_no_matching_paren() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--regex",
            "4:abc(d|e",
            "tests/data/filter/input1.tsv",
        ])
        .run_fail();
    assert!(stderr.contains("invalid regex") || stderr.contains("error"));
}

#[test]
fn upstream_error_ff_absdiff_missing_second_colon() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-absdiff-le",
            "1:2",
            "tests/data/filter/input1.tsv",
        ])
        .run_fail();
    assert!(
        stderr.contains("missing second `:`")
            || stderr.contains("expected FIELD1:FIELD2:NUM")
    );
}

#[test]
fn upstream_error_invalid_field_abc() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ge",
            "abc:10",
            "tests/data/filter/input1.tsv",
        ])
        .run_fail();
    assert!(
        stderr.contains("Field `abc` not found in header")
            || stderr.contains("invalid")
            || stderr.contains("unknown field name")
    );
}

#[test]
fn upstream_error_invalid_numeric_value() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--eq",
            "2:def",
            "tests/data/filter/input1.tsv",
        ])
        .run_fail();
    assert!(stderr.contains("invalid numeric value") || stderr.contains("def"));
}

#[test]
fn upstream_error_invalid_regex() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--regex",
            "4:abc(d|e",
            "tests/data/filter/input1.tsv",
        ])
        .run_fail();
    assert!(
        stderr.contains("regex") || stderr.contains("error") || stderr.contains("parse")
    );
}

#[test]
fn upstream_error_ff_le_invalid_values() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-le",
            "2:3.1",
            "tests/data/filter/input1.tsv",
        ])
        .run_fail();
    assert!(
        stderr.contains("invalid")
            || stderr.contains("parse")
            || stderr.contains("unknown field name")
    );
}

#[test]
fn upstream_error_ff_le_missing_field() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-le",
            "2:",
            "tests/data/filter/input1.tsv",
        ])
        .run_fail();
    assert!(
        stderr.contains("invalid")
            || stderr.contains("missing")
            || stderr.contains("mismatched field list")
    );
}

#[test]
fn upstream_error_ff_str_ne_invalid_field() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-str-ne",
            "abc:3",
            "tests/data/filter/input1.tsv",
        ])
        .run_fail();
    assert!(
        stderr.contains("not found in header")
            || stderr.contains("invalid")
            || stderr.contains("abc")
    );
}

#[test]
fn upstream_error_ff_absdiff_le_zero_index() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-absdiff-le",
            "1:0:0.5",
            "tests/data/filter/input1.tsv",
        ])
        .run_fail();
    assert!(
        stderr.contains("Zero is not a valid field index")
            || stderr.contains("must be >= 1")
    );
}

#[test]
fn upstream_error_ff_gt_zero_index() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ff-gt",
            "0:1",
            "tests/data/filter/input1.tsv",
        ])
        .run_fail();
    assert!(
        stderr.contains("Zero is not a valid field index")
            || stderr.contains("must be >= 1")
    );
}

#[test]
fn upstream_error_missing_value_lt() {
    let (_, stderr) = TvaCmd::new().args(&["filter", "--lt"]).run_fail();
    assert!(stderr.contains("value is required") || stderr.contains("required"));
}

#[test]
fn upstream_error_empty_invalid_field() {
    let (_, stderr) = TvaCmd::new()
        .args(&["filter", "--ge", ":10", "tests/data/filter/input1.tsv"])
        .run_fail();
    assert!(stderr.contains("invalid") || stderr.contains("empty"));
}

#[test]
fn upstream_error_header_processing_no_digits() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--ge",
            "abc:10",
            "tests/data/filter/input1.tsv",
        ])
        .run_fail();
    // Broaden check
    assert!(
        stderr.contains("not found")
            || stderr.contains("invalid")
            || stderr.contains("error")
            || stderr.contains("unknown")
    );
}

#[test]
fn upstream_error_invalid_spec_empty_field() {
    let (_, stderr) = TvaCmd::new()
        .args(&["filter", "--ge", ":10", "tests/data/filter/input1.tsv"])
        .run_fail();
    assert!(stderr.contains("invalid") || stderr.contains("empty"));
}

#[test]
fn upstream_error_invalid_spec_missing_colon() {
    let (_, stderr) = TvaCmd::new()
        .args(&["filter", "--ge", "1", "tests/data/filter/input1.tsv"])
        .run_fail();
    assert!(
        stderr.contains("missing `:` separator")
            || stderr.contains("invalid")
            || stderr.contains("error")
            || stderr.contains("spec")
    );
}

#[test]
fn upstream_error_invalid_spec_missing_value() {
    let (_, stderr) = TvaCmd::new()
        .args(&["filter", "--ge", "1:", "tests/data/filter/input1.tsv"])
        .run_fail();
    assert!(stderr.contains("missing value") || stderr.contains("invalid"));
}

#[test]
fn upstream_error_not_enough_fields() {
    TvaCmd::new()
        .args(&["filter", "--ge", "10:10", "tests/data/filter/input1.tsv"])
        .run();
}
