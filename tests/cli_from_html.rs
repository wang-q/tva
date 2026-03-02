#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}

#[test]
fn from_html_pup_attr() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "from",
            "html",
            "--query",
            "a attr{href}",
            "tests/data/from_html/basic.html",
        ])
        .run();
    let stdout = normalize_newlines(&stdout);

    let expected = "/a1\n/a2\n";
    assert_eq!(stdout, expected);
}

#[test]
fn from_html_pup_text() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "from",
            "html",
            "--query",
            "#content .article text{}",
            "tests/data/from_html/basic.html",
        ])
        .run();
    let stdout = normalize_newlines(&stdout);

    let expected = "Hello World\n";
    assert_eq!(stdout, expected);
}

#[test]
fn from_html_query_attr_missing_default_keeps_empty() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "from",
            "html",
            "--query",
            "a attr{title}",
            "tests/data/from_html/basic.html",
        ])
        .run();
    let stdout = normalize_newlines(&stdout);
    let expected = "\n\n";
    assert_eq!(stdout, expected);
}

#[test]
fn from_html_table_mode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "from",
            "html",
            "--table=table#t1",
            "tests/data/from_html/basic.html",
        ])
        .run();
    let stdout = normalize_newlines(&stdout);

    let expected = "ID\tName\n1\tAlice\n2\tBob\n";
    assert_eq!(stdout, expected);
}

#[test]
fn from_html_index_only_implies_table() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "from",
            "html",
            "--index",
            "1",
            "tests/data/from_html/basic.html",
        ])
        .run();
    let stdout = normalize_newlines(&stdout);

    let expected = "ID\tName\n1\tAlice\n2\tBob\n";
    assert_eq!(stdout, expected);
}

#[test]
fn from_html_table_mode_default_selector_does_not_consume_infile() {
    let (stdout, _) = TvaCmd::new()
        .args(&["from", "html", "--table", "tests/data/from_html/basic.html"])
        .run();
    let stdout = normalize_newlines(&stdout);

    let expected = "ID\tName\n1\tAlice\n2\tBob\n";
    assert_eq!(stdout, expected);
}

#[test]
fn from_html_list_mode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "from",
            "html",
            "--row",
            ".item",
            "--col",
            "Title:h2 text()",
            "--col",
            "Price:.price text()",
            "tests/data/from_html/basic.html",
        ])
        .run();
    let stdout = normalize_newlines(&stdout);

    let expected = "Title\tPrice\nApple\t$1\nBanana\t$2\n";
    assert_eq!(stdout, expected);
}

#[test]
fn from_html_outfile_writes_file_for_query() {
    use std::fs;
    use tempfile::NamedTempFile;

    let file = NamedTempFile::new().unwrap();
    let path = file.path().to_str().unwrap().to_string();

    let (_stdout, _stderr) = TvaCmd::new()
        .args(&[
            "from",
            "html",
            "--query",
            "a attr{href}",
            "-o",
            &path,
            "tests/data/from_html/basic.html",
        ])
        .run();

    let content = fs::read_to_string(&path).unwrap();
    let content = normalize_newlines(&content);
    let expected = "/a1\n/a2\n";
    assert_eq!(content, expected);
}

#[test]
fn from_html_list_mode_curly() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "from",
            "html",
            "--row",
            ".item",
            "--col",
            "Title:h2 text{}",
            "--col",
            "Price:.price text{}",
            "tests/data/from_html/basic.html",
        ])
        .run();
    let stdout = normalize_newlines(&stdout);

    let expected = "Title\tPrice\nApple\t$1\nBanana\t$2\n";
    assert_eq!(stdout, expected);
}

#[test]
fn from_html_table_and_query_conflict_should_fail() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "from",
            "html",
            "--table",
            "--query",
            "a attr{href}",
            "tests/data/from_html/basic.html",
        ])
        .run_fail();

    let stderr = normalize_newlines(&stderr);
    assert!(stderr.contains("--table"));
    assert!(stderr.contains("--query"));
}

#[test]
fn from_html_list_mode_missing_col_keeps_empty() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "from",
            "html",
            "--row",
            ".item",
            "--col",
            "Title:h2",
            "--col",
            "Missing:.not-exist",
            "--col",
            "Attr:h2 attr{not-exist}",
            "tests/data/from_html/basic.html",
        ])
        .run();
    let stdout = normalize_newlines(&stdout);

    // Header + 2 rows. Each row has 3 columns.
    // Columns 2 and 3 should be empty.
    let expected = "Title\tMissing\tAttr\nApple\t\t\nBanana\t\t\n";
    assert_eq!(stdout, expected);
}
