#[macro_use]
extern crate assert_cmd;

use std::process::Command;

#[test]
fn slice_keep_single_range() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "h1\nr1\nr2\nr3\nr4\nr5")?;
    let path = file.path().to_str().unwrap();

    // Keep rows 2-4 (r2, r3, r4)
    let output = cmd.arg("slice").arg("-r").arg("3-5").arg(path).output()?;

    // Original line numbers:
    // 1: h1
    // 2: r1
    // 3: r2
    // 4: r3
    // 5: r4
    // 6: r5

    let expected = "r2\nr3\nr4\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn slice_keep_multiple_ranges() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "1\n2\n3\n4\n5\n6\n7\n8\n9\n10")?;
    let path = file.path().to_str().unwrap();

    // Keep 1-3 and 8-10
    let output = cmd
        .arg("slice")
        .arg("-r")
        .arg("1-3")
        .arg("-r")
        .arg("8-10")
        .arg(path)
        .output()?;

    let expected = "1\n2\n3\n8\n9\n10\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn slice_drop_single_row() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "1\n2\n3\n4\n5")?;
    let path = file.path().to_str().unwrap();

    // Drop row 3
    let output = cmd
        .arg("slice")
        .arg("-r")
        .arg("3")
        .arg("--invert")
        .arg(path)
        .output()?;

    let expected = "1\n2\n4\n5\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn slice_keep_header_drop_range() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "Header\nData1\nData2\nData3\nData4")?;
    let path = file.path().to_str().unwrap();

    // Drop rows 1-3 (Header, Data1, Data2) but keep header with -H
    // So result should be: Header, Data3, Data4
    let output = cmd
        .arg("slice")
        .arg("-r")
        .arg("1-3")
        .arg("--invert")
        .arg("--header")
        .arg(path)
        .output()?;

    let expected = "Header\nData3\nData4\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn slice_keep_header_keep_range() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "Header\nData1\nData2\nData3\nData4")?;
    let path = file.path().to_str().unwrap();

    // Keep rows 4-5 (Data3, Data4) plus Header
    let output = cmd
        .arg("slice")
        .arg("-r")
        .arg("4-5")
        .arg("--header")
        .arg(path)
        .output()?;

    let expected = "Header\nData3\nData4\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn slice_open_ranges() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "1\n2\n3\n4\n5")?;
    let path = file.path().to_str().unwrap();

    // 4- (4, 5)
    let output = cmd.arg("slice").arg("-r").arg("4-").arg(path).output()?;

    let expected = "4\n5\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn slice_start_ranges() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "1\n2\n3\n4\n5")?;
    let path = file.path().to_str().unwrap();

    // -2 (1, 2)
    let output = cmd.arg("slice").arg("-r").arg("-2").arg(path).output()?;

    let expected = "1\n2\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}
