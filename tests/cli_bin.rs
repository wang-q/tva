#[macro_use]
extern crate assert_cmd;

#[test]
fn bin_basic_numeric() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "10.5\n12.8\n25.0\n10.1\n18.5")?;
    let path = file.path().to_str().unwrap();

    let output = cmd
        .arg("bin")
        .arg("--width")
        .arg("10")
        .arg("--field")
        .arg("1")
        .arg(path)
        .output()?;
    
    let expected = "10\n10\n20\n10\n10\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn bin_header_named() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "Price\n10.5\n25.0")?;
    let path = file.path().to_str().unwrap();

    let output = cmd
        .arg("bin")
        .arg("--header")
        .arg("--width")
        .arg("10")
        .arg("--field")
        .arg("Price")
        .arg(path)
        .output()?;
    
    let expected = "Price\n10\n20\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn bin_min_offset() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "12\n18\n23")?; // Bins: 5-15, 15-25
    let path = file.path().to_str().unwrap();

    let output = cmd
        .arg("bin")
        .arg("--width")
        .arg("10")
        .arg("--min")
        .arg("5")
        .arg("--field")
        .arg("1")
        .arg(path)
        .output()?;
    
    // 12 -> (12-5)/10 = 0.7 -> floor 0 -> 0*10+5 = 5
    // 18 -> (18-5)/10 = 1.3 -> floor 1 -> 1*10+5 = 15
    // 23 -> (23-5)/10 = 1.8 -> floor 1 -> 1*10+5 = 15
    let expected = "5\n15\n15\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn bin_multi_column() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "A\t12\nB\t25")?;
    let path = file.path().to_str().unwrap();

    let output = cmd
        .arg("bin")
        .arg("--width")
        .arg("10")
        .arg("--field")
        .arg("2")
        .arg(path)
        .output()?;
    
    let expected = "A\t10\nB\t20\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn bin_new_name() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "Price\n10.5\n25.0")?;
    let path = file.path().to_str().unwrap();

    let output = cmd
        .arg("bin")
        .arg("--header")
        .arg("--width")
        .arg("10")
        .arg("--field")
        .arg("Price")
        .arg("--new-name")
        .arg("Price_bin")
        .arg(path)
        .output()?;
    
    let expected = "Price\tPrice_bin\n10.5\t10\n25.0\t20\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}
