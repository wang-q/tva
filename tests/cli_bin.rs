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
