use assert_cmd::cargo::cargo_bin_cmd;
use calamine::{open_workbook_auto, Data, Reader};
use tempfile::Builder;

#[test]
fn to_xlsx_basic() -> Result<(), Box<dyn std::error::Error>> {
    let infile = "tests/data/xlsx/mcox.05.result.tsv";
    let outfile = Builder::new().suffix(".xlsx").tempfile()?;
    let outfile_path = outfile.path().to_str().unwrap();

    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("to")
        .arg("xlsx")
        .arg(infile)
        .arg("--outfile")
        .arg(outfile_path)
        .assert()
        .success();

    let mut workbook =
        open_workbook_auto(outfile_path).map_err(|e| format!("{:?}", e))?;
    let range = workbook
        .worksheet_range("mcox.05.result")
        .map_err(|e| format!("{:?}", e))?;

    assert_eq!(
        range.get_value((0, 0)),
        Some(&Data::String("#marker".to_string()))
    );

    assert_eq!(
        range.get_value((1, 0)),
        Some(&Data::String("m01+m02".to_string()))
    );
    assert_eq!(
        range.get_value((1, 1)),
        Some(&Data::String("-1.064,2.387".to_string()))
    );

    Ok(())
}

#[test]
fn to_xlsx_stdout_error() {
    let infile = "tests/data/xlsx/mcox.05.result.tsv";
    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("to")
        .arg("xlsx")
        .arg(infile)
        .arg("--outfile")
        .arg("stdout")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Output to stdout is not supported",
        ));
}

#[test]
fn to_xlsx_conditional_formatting() -> Result<(), Box<dyn std::error::Error>> {
    let infile = "tests/data/xlsx/mcox.05.result.tsv";
    let outfile = Builder::new().suffix(".xlsx").tempfile()?;
    let outfile_path = outfile.path().to_str().unwrap();

    let mut cmd = cargo_bin_cmd!("tva");
    cmd.arg("to")
        .arg("xlsx")
        .arg(infile)
        .arg("--outfile")
        .arg(outfile_path)
        .arg("-H")
        // These conditions won't match anything in the sample file because columns 2 and 3 are strings with commas
        .arg("--le")
        .arg("2:0.5")
        .arg("--str-in-fld")
        .arg("1:m01")
        .assert()
        .success();

    let mut workbook =
        open_workbook_auto(outfile_path).map_err(|e| format!("{:?}", e))?;
    let range = workbook
        .worksheet_range("mcox.05.result")
        .map_err(|e| format!("{:?}", e))?;
    assert!(range.height() > 0);

    Ok(())
}
