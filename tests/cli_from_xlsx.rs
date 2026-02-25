#[test]
fn from_xlsx_formats_file_default() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "tests/data/xlsx/formats.xlsx";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd.arg("from")
        .arg("xlsx")
        .arg(file_path)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone())?;
    let lines: Vec<&str> = stdout.lines().collect();

    // Check line count (allowing for trailing empty line handling difference)
    // The Perl test expects 13 lines.
    // Our previous run output:
    // This workbook ...
    // ...
    // Miscellaneous
    // (empty line)
    // The `lines()` iterator in Rust handles `\n` or `\r\n`.
    // If output ends with `\n`, `lines()` will return lines excluding the final empty string if `\n` is the last char? No.
    // "a\nb\n".lines() -> ["a", "b"]
    // So if there are 13 lines of text, `lines().count()` should be 13.
    assert_eq!(lines.len(), 13);
    assert!(lines[0].contains("This workbook"));

    Ok(())
}

#[test]
fn from_xlsx_formats_file_sheet_borders() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "tests/data/xlsx/formats.xlsx";

    let mut cmd = cargo_bin_cmd!("tva");
    let output = cmd.arg("from")
        .arg("xlsx")
        .arg(file_path)
        .arg("--sheet")
        .arg("Borders")
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone())?;
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines.len(), 37);
    assert!(lines[0].contains("Index\tIndex"));
    // Perl regex: qr{\t{5,}}
    // Check if second line has at least 5 tabs.
    let tab_count = lines[1].chars().filter(|&c| c == '\t').count();
    assert!(tab_count >= 5);

    Ok(())
}

#[test]
fn from_xlsx_error_missing_file() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("from")
        .arg("xlsx")
        .assert()
        .failure()
        .stderr(predicates::str::contains("Usage: tva from xlsx"));
}

#[test]
fn from_xlsx_error_non_existent_file() {
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("from")
        .arg("xlsx")
        .arg("tests/data/xlsx/non_existent.xlsx")
        .assert()
        .failure()
        .stderr(predicates::str::contains("Failed to open file"));
}
