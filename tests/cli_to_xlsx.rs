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

#[test]
fn to_xlsx_default_outfile() {
    let infile = "tests/data/xlsx/mcox.05.result.tsv";

    // We need to run this in a temp dir to avoid polluting source tree
    let temp_dir = tempfile::tempdir().unwrap();
    let infile_path = temp_dir.path().join("mcox.05.result.tsv");
    std::fs::copy(infile, &infile_path).unwrap();

    // Run without --outfile
    TvaCmd::new()
        .current_dir(temp_dir.path())
        .args(&["to", "xlsx", "mcox.05.result.tsv"])
        .run();

    let expected_outfile = temp_dir.path().join("mcox.05.result.xlsx");
    assert!(expected_outfile.exists());
}

#[test]
fn to_xlsx_sheet_name() {
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
            "--sheet",
            "CustomSheet",
        ])
        .run();

    let mut workbook =
        open_workbook_auto(outfile_path).expect("Failed to open workbook");
    // Verify sheet exists
    let _ = workbook
        .worksheet_range("CustomSheet")
        .expect("Sheet CustomSheet not found");
}

#[test]
fn to_xlsx_write_numeric_and_string() {
    let temp_dir = tempfile::tempdir().unwrap();
    let infile_path = temp_dir.path().join("data.tsv");
    // Row 1: Number, String, Mixed
    std::fs::write(&infile_path, "1.23\thello\t1.2.3\n").unwrap();

    let outfile_path = temp_dir.path().join("out.xlsx");

    TvaCmd::new()
        .args(&[
            "to",
            "xlsx",
            infile_path.to_str().unwrap(),
            "--outfile",
            outfile_path.to_str().unwrap(),
        ])
        .run();

    let mut workbook = open_workbook_auto(outfile_path.to_str().unwrap()).unwrap();
    let range = workbook.worksheet_range("data").unwrap();

    // 1.23 should be float
    match range.get_value((0, 0)) {
        Some(Data::Float(f)) => assert!((f - 1.23).abs() < 1e-10),
        v => panic!("Expected Float(1.23), got {:?}", v),
    }

    // hello should be string
    assert_eq!(
        range.get_value((0, 1)),
        Some(&Data::String("hello".to_string()))
    );

    // 1.2.3 should be string (parse failed)
    assert_eq!(
        range.get_value((0, 2)),
        Some(&Data::String("1.2.3".to_string()))
    );
}

#[test]
fn to_xlsx_conditional_formatting_ge_bt() {
    let temp_dir = tempfile::tempdir().unwrap();
    let infile_path = temp_dir.path().join("data.tsv");
    std::fs::write(&infile_path, "Header\n10\n20\n30\n").unwrap();
    let outfile_path = temp_dir.path().join("out.xlsx");

    TvaCmd::new()
        .args(&[
            "to",
            "xlsx",
            infile_path.to_str().unwrap(),
            "--outfile",
            outfile_path.to_str().unwrap(),
            "--header",
            "--ge",
            "1:15", // col 1 >= 15
            "--bt",
            "1:10:25", // 10 <= col 1 <= 25
        ])
        .run();

    assert!(outfile_path.exists());
}
