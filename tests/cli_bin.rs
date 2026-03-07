#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn bin_basic_numeric() -> anyhow::Result<()> {
    let input = "10.5\n12.8\n25.0\n10.1\n18.5";
    let expected = "10\n10\n20\n10\n10\n";

    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&["bin", "--width", "10", "--field", "1"])
        .run();

    assert_eq!(stdout.replace("\r\n", "\n"), expected);
    Ok(())
}

#[test]
fn bin_header_named() -> anyhow::Result<()> {
    let input = "Price\n10.5\n25.0";
    let expected = "Price\n10\n20\n";

    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&["bin", "--header", "--width", "10", "--field", "Price"])
        .run();

    assert_eq!(stdout.replace("\r\n", "\n"), expected);
    Ok(())
}

#[test]
fn bin_min_offset() -> anyhow::Result<()> {
    let input = "12\n18\n23"; // Bins: 5-15, 15-25

    // 12 -> (12-5)/10 = 0.7 -> floor 0 -> 0*10+5 = 5
    // 18 -> (18-5)/10 = 1.3 -> floor 1 -> 1*10+5 = 15
    // 23 -> (23-5)/10 = 1.8 -> floor 1 -> 1*10+5 = 15
    let expected = "5\n15\n15\n";

    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&["bin", "--width", "10", "--min", "5", "--field", "1"])
        .run();

    assert_eq!(stdout.replace("\r\n", "\n"), expected);
    Ok(())
}

#[test]
fn bin_multi_column() -> anyhow::Result<()> {
    let input = "A\t12\nB\t25";
    let expected = "A\t10\nB\t20\n";

    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&["bin", "--width", "10", "--field", "2"])
        .run();

    assert_eq!(stdout.replace("\r\n", "\n"), expected);
    Ok(())
}

#[test]
fn bin_new_name() -> anyhow::Result<()> {
    let input = "Price\n10.5\n25.0";
    let expected = "Price\tPrice_bin\n10.5\t10\n25.0\t20\n";

    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&[
            "bin",
            "--header",
            "--width",
            "10",
            "--field",
            "Price",
            "--new-name",
            "Price_bin",
        ])
        .run();

    assert_eq!(stdout.replace("\r\n", "\n"), expected);
    Ok(())
}

#[test]
fn bin_error_width_non_positive() {
    let (_, stderr) = TvaCmd::new()
        .args(&["bin", "--width", "0", "--field", "1"])
        .stdin("10\n")
        .run_fail();
    assert!(stderr.contains("Width must be positive"));

    let (_, stderr2) = TvaCmd::new()
        .args(&["bin", "--width=-5", "--field", "1"])
        .stdin("10\n")
        .run_fail();
    assert!(stderr2.contains("Width must be positive"));
}

#[test]
fn bin_error_field_name_requires_header() {
    let (_, stderr) = TvaCmd::new()
        .args(&["bin", "--width", "10", "--field", "Price"])
        .stdin("10\n")
        .run_fail();
    assert!(stderr.contains("Field name 'Price' requires --header"));
}

#[test]
fn bin_error_field_not_found_in_header() {
    let (_, stderr) = TvaCmd::new()
        .args(&["bin", "--width", "10", "--field", "Missing", "--header"])
        .stdin("Price\n10\n")
        .run_fail();
    assert!(stderr.contains("Field 'Missing' not found in header"));
}

#[test]
fn bin_new_name_field_index_out_of_bounds() {
    let input = "10\n20\n";
    let expected = "10\t\n20\t\n";

    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&["bin", "--width", "10", "--field", "2", "--new-name", "Bin"])
        .run();

    assert_eq!(stdout.replace("\r\n", "\n"), expected);
}

#[test]
fn bin_multiple_files_header() {
    let file1 = "H\n10\n";
    let file2 = "H\n20\n";

    let temp_dir = tempfile::tempdir().unwrap();
    let p1 = temp_dir.path().join("f1.tsv");
    let p2 = temp_dir.path().join("f2.tsv");
    std::fs::write(&p1, file1).unwrap();
    std::fs::write(&p2, file2).unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "bin",
            "--width",
            "10",
            "--field",
            "1",
            "--header",
            p1.to_str().unwrap(),
            p2.to_str().unwrap(),
        ])
        .run();

    // Expected: H\n10\n20\n (bins: 10->10, 20->20)
    assert_eq!(stdout.replace("\r\n", "\n"), "H\n10\n20\n");
}

#[test]
fn bin_field_index_zero_fail() {
    let (_, stderr) = TvaCmd::new()
        .args(&["bin", "--width", "10", "--field", "0"])
        .stdin("10\n")
        .run_fail();
    assert!(stderr.contains("Field index must be >= 1"));
}

#[test]
fn bin_field_index_logic_error_unreachable() {
    // Tests L139-140: Field index logic error
    // This is technically unreachable if validation works, but we can try to trigger it
    // if `field_idx` remains None after all checks.
    // But `field_idx` is set if field is numeric OR if header resolution succeeds.
    // If header resolution fails, it returns error earlier.
    // So this is defensive coding. Hard to trigger in CLI test without mocking internal state.
}

#[test]
fn bin_new_name_field_parsing_optimization() {
    // We use a file with multiple columns and select a middle one.
    let input = "A\t12\tC\nB\t25\tD";
    let expected = "A\t12\tC\t10\nB\t25\tD\t20\n"; // Binning col 2

    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&["bin", "--width", "10", "--field", "2", "--new-name", "Bin"])
        .run();

    assert_eq!(stdout.replace("\r\n", "\n"), expected);
}

#[test]
fn bin_replace_mode_non_numeric_fallback() {
    let input = "10\nNotANum\n30";
    let expected = "10\nNotANum\n30\n";

    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&["bin", "--width", "10", "--field", "1"])
        .run();

    assert_eq!(stdout.replace("\r\n", "\n"), expected);
}
