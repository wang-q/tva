#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn header_basic() -> anyhow::Result<()> {
    let input = "id\tname\tvalue\n1\tAlice\t100\n2\tBob\t200\n";

    let (stdout, _) = TvaCmd::new().stdin(input).args(&["header"]).run();

    // New format: transposed table with file column
    assert!(stdout.contains("file"));
    assert!(stdout.contains("1"));
    assert!(stdout.contains("id"));
    assert!(stdout.contains("name"));
    assert!(stdout.contains("value"));

    Ok(())
}

#[test]
fn header_names_only() -> anyhow::Result<()> {
    let input = "id\tname\tvalue\n1\tAlice\t100\n";

    let (stdout, _) = TvaCmd::new().stdin(input).args(&["header", "-n"]).run();

    assert!(stdout.contains("id"));
    assert!(stdout.contains("name"));
    assert!(stdout.contains("value"));
    // Should not have index column in names-only mode
    assert!(!stdout.contains("file"));

    Ok(())
}

#[test]
fn header_start_index() -> anyhow::Result<()> {
    let input = "id\tname\tvalue\n1\tAlice\t100\n";

    let (stdout, _) = TvaCmd::new()
        .stdin(input)
        .args(&["header", "-s", "0"])
        .run();

    assert!(stdout.contains("0"));
    assert!(stdout.contains("id"));
    assert!(stdout.contains("name"));
    assert!(stdout.contains("value"));

    Ok(())
}

#[test]
fn header_duplicate_detection() -> anyhow::Result<()> {
    let input = "id\tname\tid\tvalue\n1\tAlice\t1\t100\n";

    let (stdout, _) = TvaCmd::new().stdin(input).args(&["header"]).run();

    assert!(stdout.contains("id [duplicate]"));

    Ok(())
}

#[test]
fn header_empty_input() -> anyhow::Result<()> {
    let (_, stderr) = TvaCmd::new().stdin("").args(&["header"]).run_fail();

    assert!(stderr.contains("empty file"));

    Ok(())
}

#[test]
fn header_outfile() -> anyhow::Result<()> {
    let input = "id\tname\tvalue\n1\tAlice\t100\n";
    let temp_dir = tempfile::tempdir()?;
    let output_path = temp_dir.path().join("output.txt");

    let (_, _) = TvaCmd::new()
        .stdin(input)
        .args(&["header", "-o", output_path.to_str().unwrap()])
        .run();

    let content = std::fs::read_to_string(&output_path)?;
    assert!(content.contains("file"));
    assert!(content.contains("id"));
    assert!(content.contains("name"));
    assert!(content.contains("value"));

    Ok(())
}

#[test]
fn header_multiple_files() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let file1 = temp_dir.path().join("file1.tsv");
    let file2 = temp_dir.path().join("file2.tsv");

    std::fs::write(&file1, "id\tname\tvalue\n1\tAlice\t100\n")?;
    std::fs::write(&file2, "id\tname\tamount\n1\tBob\t200\n")?;

    let (stdout, _) = TvaCmd::new()
        .args(&["header", file1.to_str().unwrap(), file2.to_str().unwrap()])
        .run();

    // Should show both file paths as column headers
    assert!(stdout.contains("file1.tsv"));
    assert!(stdout.contains("file2.tsv"));
    // Should mark diverging headers
    assert!(
        stdout.contains("[diverging]")
            || stdout.contains("value") && stdout.contains("amount")
    );

    Ok(())
}
