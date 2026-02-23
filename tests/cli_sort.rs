use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn sort_default_lexicographic_single_key() -> anyhow::Result<()> {
    let input = "a\t2\nc\t1\nb\t3\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sort")
        .arg("-k")
        .arg("1")
        .write_stdin(input)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, "a\t2\nb\t3\nc\t1\n");

    Ok(())
}

#[test]
fn sort_numeric_reverse_single_key() -> anyhow::Result<()> {
    let input = "a\t2\nc\t10\nb\t3\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sort")
        .arg("-k")
        .arg("2")
        .arg("-n")
        .arg("-r")
        .write_stdin(input)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, "c\t10\nb\t3\na\t2\n");

    Ok(())
}

#[test]
fn sort_multiple_keys() -> anyhow::Result<()> {
    let input = "a\t2\nc\t1\nb\t1\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sort")
        .arg("-k")
        .arg("2,1")
        .write_stdin(input)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, "b\t1\nc\t1\na\t2\n");

    Ok(())
}

#[test]
fn sort_default_all_columns_when_no_key() -> anyhow::Result<()> {
    let input = "b\t2\nb\t1\na\t3\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd.arg("sort").write_stdin(input).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, "a\t3\nb\t1\nb\t2\n");

    Ok(())
}

#[test]
fn sort_respects_custom_delimiter() -> anyhow::Result<()> {
    let input = "a,2\nc,1\nb,3\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sort")
        .arg("-t")
        .arg(",")
        .arg("-k")
        .arg("1")
        .write_stdin(input)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, "a,2\nb,3\nc,1\n");

    Ok(())
}

#[test]
fn sort_numeric_with_non_numeric_values() -> anyhow::Result<()> {
    let input = "x\n10\nLETTER\n2\n1\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sort")
        .arg("-k")
        .arg("1")
        .arg("-n")
        .write_stdin(input)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, "x\nLETTER\n1\n2\n10\n");

    Ok(())
}

#[test]
fn sort_reverse_lexicographic_single_key() -> anyhow::Result<()> {
    let input = "a\t2\nc\t1\nb\t3\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sort")
        .arg("-k")
        .arg("1")
        .arg("-r")
        .write_stdin(input)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, "c\t1\nb\t3\na\t2\n");

    Ok(())
}

#[test]
fn sort_lexicographic_file_names() -> anyhow::Result<()> {
    let input = "file2.txt\na\nfile10.txt\nfile1.txt\n";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("sort")
        .arg("-k")
        .arg("1")
        .write_stdin(input)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, "a\nfile1.txt\nfile10.txt\nfile2.txt\n");

    Ok(())
}
