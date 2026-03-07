#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;
use rust_xlsxwriter::Workbook;
use tempfile::Builder;

#[test]
fn from_xlsx_basic() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary XLSX file
    let file = Builder::new().suffix(".xlsx").tempfile()?;
    let file_path = file.path().to_str().unwrap();

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.write(0, 0, "Header1")?;
    worksheet.write(0, 1, "Header2")?;
    worksheet.write(1, 0, "Value1")?;
    worksheet.write(1, 1, 123)?;
    worksheet.write(2, 0, "Value2")?;
    worksheet.write(2, 1, 45.6)?;
    workbook.save(file_path)?;

    // Run tva from xlsx
    let (stdout, _) = TvaCmd::new().args(&["from", "xlsx", file_path]).run();
    assert_eq!(stdout, "Header1\tHeader2\nValue1\t123\nValue2\t45.6\n");

    Ok(())
}

#[test]
fn from_xlsx_list_sheets() -> Result<(), Box<dyn std::error::Error>> {
    let file = Builder::new().suffix(".xlsx").tempfile()?;
    let file_path = file.path().to_str().unwrap();

    let mut workbook = Workbook::new();
    let _ = workbook.add_worksheet().set_name("Sheet1")?;
    let _ = workbook.add_worksheet().set_name("Sheet2")?;
    workbook.save(file_path)?;

    let (stdout, _) = TvaCmd::new()
        .args(&["from", "xlsx", "--list-sheets", file_path])
        .run();
    assert_eq!(stdout, "Sheet1\nSheet2\n");

    Ok(())
}

#[test]
fn from_xlsx_specific_sheet() -> Result<(), Box<dyn std::error::Error>> {
    let file = Builder::new().suffix(".xlsx").tempfile()?;
    let file_path = file.path().to_str().unwrap();

    let mut workbook = Workbook::new();
    {
        let sheet1 = workbook.add_worksheet().set_name("Sheet1")?;
        sheet1.write(0, 0, "S1")?;
    }

    {
        let sheet2 = workbook.add_worksheet().set_name("Sheet2")?;
        sheet2.write(0, 0, "S2")?;
    }

    workbook.save(file_path)?;

    let (stdout, _) = TvaCmd::new()
        .args(&["from", "xlsx", "--sheet", "Sheet2", file_path])
        .run();
    assert_eq!(stdout, "S2\n");

    Ok(())
}

#[test]
fn from_xlsx_formats_file_default() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "tests/data/xlsx/formats.xlsx";

    let (stdout, _) = TvaCmd::new().args(&["from", "xlsx", file_path]).run();
    let lines: Vec<&str> = stdout.lines().collect();

    // Check line count (allowing for trailing empty line handling difference)
    // The Perl test expects 13 lines.
    assert_eq!(lines.len(), 13);
    assert!(lines[0].contains("This workbook"));

    Ok(())
}

#[test]
fn from_xlsx_formats_file_sheet_borders() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "tests/data/xlsx/formats.xlsx";

    let (stdout, _) = TvaCmd::new()
        .args(&["from", "xlsx", file_path, "--sheet", "Borders"])
        .run();
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
fn from_xlsx_formats_file_list_sheets() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "tests/data/xlsx/formats.xlsx";

    let (stdout, _) = TvaCmd::new()
        .args(&["from", "xlsx", "--list-sheets", file_path])
        .run();
    assert!(stdout.contains("Introduction\nFonts\nNamed colors\nStandard colors\nNumeric formats\nBorders\nPatterns\nAlignment\nMiscellaneous\n"));

    Ok(())
}

#[test]
fn from_xlsx_error_missing_file() {
    let (_stdout, stderr) = TvaCmd::new().args(&["from", "xlsx"]).run_fail();
    assert!(stderr.contains("Usage:"));
}

#[test]
fn from_xlsx_error_non_existent_file() {
    let (_stdout, stderr) = TvaCmd::new()
        .args(&["from", "xlsx", "tests/data/xlsx/non_existent.xlsx"])
        .run_fail();
    assert!(stderr.contains("Failed to open file"));
}

#[test]
fn from_xlsx_sheet_not_found_error() -> Result<(), Box<dyn std::error::Error>> {
    let file = Builder::new().suffix(".xlsx").tempfile()?;
    let file_path = file.path().to_str().unwrap();

    let mut workbook = Workbook::new();
    let _ = workbook.add_worksheet().set_name("Sheet1")?;
    workbook.save(file_path)?;

    let (_, stderr) = TvaCmd::new()
        .args(&["from", "xlsx", "--sheet", "NonExistent", file_path])
        .run_fail();

    assert!(stderr.contains("Failed to read sheet 'NonExistent'"));

    Ok(())
}

#[test]
fn from_xlsx_data_types() -> Result<(), Box<dyn std::error::Error>> {
    let file = Builder::new().suffix(".xlsx").tempfile()?;
    let file_path = file.path().to_str().unwrap();

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // Row 0: Various types
    worksheet.write_string(0, 0, "String")?;
    worksheet.write_number(0, 1, 123.45)?;
    worksheet.write_boolean(0, 2, true)?;
    worksheet.write_boolean(0, 3, false)?;

    // Row 1: Empty handling
    // Empty string
    worksheet.write_string(1, 0, "")?;
    // Explicit empty
    worksheet.write_string(1, 2, "AfterEmpty")?;

    workbook.save(file_path)?;

    let (stdout, _) = TvaCmd::new().args(&["from", "xlsx", file_path]).run();

    let lines: Vec<&str> = stdout.lines().collect();

    // Row 0: String, 123.45, true, false
    assert!(lines[0].contains("String"));
    assert!(lines[0].contains("123.45"));
    assert!(lines[0].contains("true"));
    assert!(lines[0].contains("false"));

    // Row 1: "", "", "AfterEmpty"
    assert!(lines[1].contains("AfterEmpty"));

    Ok(())
}
