use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn md_basic() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd.arg("md").write_stdin("H1\tH2\nA\t1\nB\t2\n").output()?;

    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    // The markdown formatter aligns columns
    let expected = "| H1  | H2  |\n| --- | --- |\n| A   | 1   |\n| B   | 2   |\n";
    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn md_center() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("md")
        .arg("--center")
        .arg("1")
        .write_stdin("H1\tH2\nA\t1\nB\t2\n")
        .output()?;

    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    let expected =
        "|  H1   | H2  |\n| :---: | --- |\n|   A   | 1   |\n|   B   | 2   |\n";
    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn md_right() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("md")
        .arg("--right")
        .arg("2")
        .write_stdin("H1\tH2\nA\t1\nB\t2\n")
        .output()?;

    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    let expected = "| H1  |   H2 |\n| --- | ---: |\n| A   |    1 |\n| B   |    2 |\n";
    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn md_num() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("md")
        .arg("--num")
        .write_stdin("H1\tH2\nA\t1\nB\t2\n")
        .output()?;

    // H2 is numeric, so it should be right-aligned
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    let expected = "| H1  |   H2 |\n| --- | ---: |\n| A   |    1 |\n| B   |    2 |\n";
    assert_eq!(stdout, expected);

    Ok(())
}

#[test]
fn md_fmt() -> anyhow::Result<()> {
    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd
        .arg("md")
        .arg("--fmt")
        .arg("--digits")
        .arg("2")
        .write_stdin("H1\tH2\nA\t1\nB\t2.567\n")
        .output()?;

    // H2 is numeric, should be right-aligned and formatted
    let stdout = String::from_utf8(output.stdout)?.replace("\r\n", "\n");
    let expected = "| H1  |   H2 |\n| --- | ---: |\n| A   | 1.00 |\n| B   | 2.57 |\n";
    assert_eq!(stdout, expected);

    Ok(())
}
