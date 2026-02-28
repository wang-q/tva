#[macro_use]
#[path = "common/mod.rs"]
mod common;

use calamine::{open_workbook_auto, Data, Reader};
use common::TvaCmd;
use tempfile::Builder;

#[test]
fn to_xlsx_basic() {
    let infile = "tests/data/xlsx/mcox.05.result.tsv";
    let outfile = Builder::new().suffix(".xlsx").tempfile().unwrap();
    let outfile_path = outfile.path().to_str().unwrap();

    TvaCmd::new()
        .args(&["to", "xlsx", infile, "--outfile", outfile_path])
        .run();

    let mut workbook =
        open_workbook_auto(outfile_path).expect("Failed to open workbook");
    let range = workbook
        .worksheet_range("mcox.05.result")
        .expect("Failed to get worksheet");

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
}

#[test]
fn to_xlsx_stdout_error() {
    let infile = "tests/data/xlsx/mcox.05.result.tsv";
    let (_, stderr) = TvaCmd::new()
        .args(&["to", "xlsx", infile, "--outfile", "stdout"])
        .run_fail();

    assert!(stderr.contains("Output to stdout is not supported"));
}

#[test]
fn to_xlsx_conditional_formatting() {
    let infile = "tests/data/xlsx/mcox.05.result.tsv";
    let outfile = Builder::new().suffix(".xlsx").tempfile().unwrap();
    let outfile_path = outfile.path().to_str().unwrap();

    TvaCmd::new()
        .args(&[
            "to",
            "xlsx",
            infile,
            "--outfile",
            outfile_path,
            "-H",
            // These conditions won't match anything in the sample file because columns 2 and 3 are strings with commas
            "--le",
            "2:0.5",
            "--str-in-fld",
            "1:m01",
        ])
        .run();

    let mut workbook =
        open_workbook_auto(outfile_path).expect("Failed to open workbook");
    let range = workbook
        .worksheet_range("mcox.05.result")
        .expect("Failed to get worksheet");
    assert!(range.height() > 0);
}
