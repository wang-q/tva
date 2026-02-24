use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn wider_basic() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "ID\tname\tvalue")?;
    writeln!(file, "A\tcost\t10")?;
    writeln!(file, "A\tsize\t5")?;
    writeln!(file, "B\tcost\t20")?;
    writeln!(file, "B\tsize\t8")?;
    let path = file.path().to_str().unwrap();

    let output = cmd
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("name")
        .arg("--values-from")
        .arg("value")
        .output()?;

    let expected = "ID\tcost\tsize\nA\t10\t5\nB\t20\t8\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn wider_missing_values() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "ID\tname\tvalue")?;
    writeln!(file, "A\tcost\t10")?;
    writeln!(file, "B\tsize\t8")?;
    let path = file.path().to_str().unwrap();

    let output = cmd
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("name")
        .arg("--values-from")
        .arg("value")
        .arg("--values-fill")
        .arg("0")
        .arg("--names-sort")
        .output()?;

    let expected = "ID\tcost\tsize\nA\t10\t0\nB\t0\t8\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn wider_explicit_id() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "ID\tDate\tname\tvalue")?;
    writeln!(file, "A\t2020\tcost\t10")?;
    writeln!(file, "A\t2021\tcost\t12")?;
    let path = file.path().to_str().unwrap();

    let output = cmd
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("name")
        .arg("--values-from")
        .arg("value")
        .arg("--id-cols")
        .arg("ID")
        .output()?;

    // With explicit ID "ID", "Date" is ignored.
    // Row 1: ID=A, name=cost, value=10
    // Row 2: ID=A, name=cost, value=12
    // Row 2 overwrites Row 1 because ID and Name are same.
    let expected = "ID\tcost\nA\t12\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn wider_doc_example_us_rent_income() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("wider")
        .arg("docs/data/us_rent_income.tsv")
        .arg("--names-from")
        .arg("variable")
        .arg("--values-from")
        .arg("estimate")
        .arg("--id-cols")
        .arg("GEOID,NAME")
        .output()?;

    let expected = "GEOID\tNAME\tincome\trent\n01\tAlabama\t24476\t747\n02\tAlaska\t32940\t1200\n04\tArizona\t27517\t972\n05\tArkansas\t23789\t709\n06\tCalifornia\t29454\t1358\n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn wider_multi_file_error() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file1 = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file1, "ID\tname\tvalue")?;
    writeln!(file1, "A\tcost\t10")?;
    
    let mut file2 = tempfile::NamedTempFile::new()?;
    // Only 2 columns, but first file had 3
    writeln!(file2, "ID\tvalue")?;
    writeln!(file2, "B\t20")?;

    let path1 = file1.path().to_str().unwrap();
    let path2 = file2.path().to_str().unwrap();

    let output = cmd
        .arg("wider")
        .arg(path1)
        .arg(path2)
        .arg("--names-from")
        .arg("name")
        .arg("--values-from")
        .arg("value")
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("All files must have the same column structure"));
    Ok(())
}

#[test]
fn wider_preserve_space() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let mut file = tempfile::NamedTempFile::new()?;
    use std::io::Write;
    writeln!(file, "ID\tname\tvalue")?;
    // Value is a space
    writeln!(file, "A\tcost\t ")?;
    let path = file.path().to_str().unwrap();

    let output = cmd
        .arg("wider")
        .arg(path)
        .arg("--names-from")
        .arg("name")
        .arg("--values-from")
        .arg("value")
        .output()?;

    let expected = "ID\tcost\nA\t \n";
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    assert_eq!(stdout, expected);
    Ok(())
}
