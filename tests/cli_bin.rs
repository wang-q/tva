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
    // Input has 1 column, but we ask for field 2
    let input = "10\n20\n";
    // With --new-name, we append a new column.
    // If field index is out of bounds, the code should gracefully handle it (skipped == idx - 1 check fails or iter ends).
    // Based on the code snippet:
    // if idx == 0 { ... } else { loop ... }
    // if skipped == idx - 1 { if let Some(start_pos) = iter.next() ... }
    // If out of bounds, field_bytes remains None.
    // Then nothing is written for the bin value?
    // Let's verify behavior. It seems it just writes empty string or nothing?
    // Wait, the code:
    // if let Some(bytes) = field_bytes { ... write!(writer, "{}", result)?; }
    // So if field_bytes is None, nothing is written for that value.
    // But we still write "\n".
    // So it should be "10\t\n20\t\n" if new-name is used?

    let expected = "10\t\n20\t\n";

    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&["bin", "--width", "10", "--field", "2", "--new-name", "Bin"])
        .run();

    assert_eq!(stdout.replace("\r\n", "\n"), expected);
}
