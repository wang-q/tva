#[macro_use]
extern crate assert_cmd;

#[test]
fn reverse_basic() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "1\n2\n3")?;
    let path = file.path().to_str().unwrap();

    let output = cmd
        .arg("reverse")
        .arg(path)
        .output()?;

    let expected = "3\n2\n1\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn reverse_header() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "H\n1\n2\n3")?;
    let path = file.path().to_str().unwrap();

    let output = cmd
        .arg("reverse")
        .arg("--header")
        .arg(path)
        .output()?;

    let expected = "H\n3\n2\n1\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn reverse_stdin() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let input = "1\n2\n3\n";

    let output = cmd
        .arg("reverse")
        .write_stdin(input)
        .output()?;

    let expected = "3\n2\n1\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}
