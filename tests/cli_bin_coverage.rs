use crate::common::TvaCmd;

#[macro_use]
#[path = "common/mod.rs"]
mod common;

#[test]
fn bin_error_width_zero() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "bin",
            "--width",
            "0.0",
            "-f",
            "1",
            "tests/data/bin/input.tsv",
        ])
        .run_fail();
    assert!(stderr.contains("Width must be positive"));
}

#[test]
fn bin_error_width_negative() {
    // To pass negative number as value, we might need --width=-1.0 or similar.
    let (_, stderr) = TvaCmd::new()
        .args(&["bin", "--width=-1.0", "-f", "1", "tests/data/bin/input.tsv"])
        .run_fail();
    assert!(
        stderr.contains("Width must be positive") || stderr.contains("invalid value"),
        "stderr was: {}",
        stderr
    );
}

#[test]
fn bin_error_field_name_requires_header() {
    // If field is non-numeric (e.g. "foo") and --header is not provided
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "bin",
            "--width",
            "10",
            "-f",
            "foo",
            "tests/data/bin/input.tsv",
        ])
        .run_fail();
    assert!(stderr.contains("Field name 'foo' requires --header"));
}

#[test]
fn bin_error_field_index_zero() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "bin",
            "--width",
            "10",
            "-f",
            "0",
            "tests/data/bin/input.tsv",
        ])
        .run_fail();
    assert!(stderr.contains("Field index must be >= 1"));
}

#[test]
fn bin_error_field_not_found_in_header() {
    // Create a temporary file with header but missing the target field
    let (_, stderr) = TvaCmd::new()
        .stdin("f1\tf2\n10\t20")
        .args(&["bin", "--width", "10", "--header", "-f", "f3"])
        .run_fail();
    assert!(stderr.contains("Field 'f3' not found in header"));
}

#[test]
fn bin_data_invalid_utf8_in_numeric_field() {
    use assert_cmd::cargo::cargo_bin_cmd;

    // Using 0xFF which is invalid UTF-8
    let input = b"10\t\xFF\t30\n";

    // assert_cmd doesn't expose underlying process builder directly in a way that allows us to
    // easily write raw bytes to stdin while capturing output, unless we use Command::cargo_bin.
    // TvaCmd uses cargo_bin_cmd! which returns assert_cmd::Command.
    // assert_cmd::Command has .write_stdin(impl AsRef<[u8]>) but we need to verify output bytes too.
    // TvaCmd wraps this but enforces String output.
    let mut cmd = cargo_bin_cmd!("tva");
    let assert = cmd
        .args(&["bin", "--width", "10", "-f", "2"])
        .write_stdin(input.as_slice())
        .assert();

    let output = assert.get_output();
    assert_eq!(output.stdout, input);
}

#[test]
fn bin_data_non_numeric_field() {
    // Field 2 is "abc", not numeric.
    // Should fallback to writing original value.
    let input = "10\tabc\t30\n";
    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&["bin", "--width", "10", "-f", "2"])
        .run();

    assert_eq!(stdout.as_bytes(), input.as_bytes());
}

#[test]
fn bin_new_name_append_mode_non_numeric() {
    let input = "10\tabc\t30\n";
    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&["bin", "--width", "10", "-f", "2", "--new-name", "binned"])
        .run();

    assert_eq!(stdout.as_bytes(), b"10\tabc\t30\t\n");
}

#[test]
fn bin_field_index_too_large() {
    let (stdout, _) = TvaCmd::new()
        .stdin("10\t20\n")
        .args(&["bin", "--width", "10", "-f", "10"])
        .run();
    assert_eq!(stdout, "10\t20\n");
}
