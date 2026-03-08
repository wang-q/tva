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

#[test]
fn from_html_query_empty_returns_original() {
    let content = "<html><body><p>Test</p></body></html>";
    let (stdout, _) = TvaCmd::new().args(&["from", "html"]).stdin(content).run();
    assert_eq!(stdout, content);
}

#[test]
fn from_html_parse_query_empty() {
    let (stdout, _) = TvaCmd::new()
        .args(&["from", "html", "--query", "   "])
        .stdin("<html></html>")
        .run();
    let content = "<html></html>";
    assert_eq!(stdout, content);
}

#[test]
fn from_html_query_attr_paren() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "from",
            "html",
            "--query",
            "a attr(href)",
            "tests/data/from_html/basic.html",
        ])
        .run();
    let stdout = normalize_newlines(&stdout);
    let expected = "/a1\n/a2\n";
    assert_eq!(stdout, expected);
}

#[test]
fn from_html_query_attr_paren_quoted() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "from",
            "html",
            "--query",
            "a attr(\"href\")",
            "tests/data/from_html/basic.html",
        ])
        .run();
    let stdout = normalize_newlines(&stdout);
    let expected = "/a1\n/a2\n";
    assert_eq!(stdout, expected);
}

#[test]
fn from_html_index_zero_fail() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "from",
            "html",
            "--index",
            "0",
            "tests/data/from_html/basic.html",
        ])
        .run_fail();
    assert!(stderr.contains("Index must be >= 1"));
}

#[test]
fn from_html_table_not_found() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "from",
            "html",
            "--table=#nonexistent",
            "tests/data/from_html/basic.html",
        ])
        .run_fail();

    if !stderr.contains("No table found matching") {
        panic!("stderr was: {}", stderr);
    }
}

#[test]
fn from_html_table_direct_tr() {
    let html = "<table><tr><td>Direct</td></tr></table>";
    let (stdout, _) = TvaCmd::new()
        .args(&["from", "html", "--table"])
        .stdin(html)
        .run();
    assert_eq!(normalize_newlines(&stdout), "Direct\n");
}

#[test]
fn from_html_table_nested_ignored() {
    let html = "<table><div>Ignored</div><tr><td>Row</td></tr></table>";
    let (stdout, _) = TvaCmd::new()
        .args(&["from", "html", "--table"])
        .stdin(html)
        .run();
    assert_eq!(normalize_newlines(&stdout), "Row\n");
}

#[test]
fn from_html_col_invalid_format() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "from",
            "html",
            "--row",
            "div",
            "--col",
            "InvalidFormat",
            "tests/data/from_html/basic.html",
        ])
        .run_fail();
    assert!(stderr.contains("Invalid column definition"));
}

#[test]
fn from_html_col_empty_selector() {
    let html = "<div class='row'>RowContent</div>";
    let (stdout, _) = TvaCmd::new()
        .args(&["from", "html", "--row", ".row", "--col", "Whole:"])
        .stdin(html)
        .run();
    assert_eq!(normalize_newlines(&stdout), "Whole\nRowContent\n");
}

#[test]
fn from_html_col_invalid_selector() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "from",
            "html",
            "--row",
            "div",
            "--col",
            "Name:!!!",
            "tests/data/from_html/basic.html",
        ])
        .run_fail();
    assert!(stderr.contains("Invalid column selector"));
}

#[test]
fn from_html_col_attr_paren_syntax() {
    let html = r#"
    <div class="item"><a href="/a1">1</a></div>
    <div class="item"><a href="/a2">2</a></div>
    "#;

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "from",
            "html",
            "--row",
            ".item",
            "--col",
            "Link:a attr(href)",
        ])
        .stdin(html)
        .run();

    let expected = "Link\n/a1\n/a2\n";
    assert_eq!(normalize_newlines(&stdout), expected);
}

#[test]
fn from_html_col_attr_invalid_syntax() {
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "from",
            "html",
            "--row",
            "div",
            "--col",
            "Name:attr{href",
            "tests/data/from_html/basic.html",
        ])
        .run_fail();
    assert!(stderr.contains("Invalid attr{} syntax"));
}

#[test]
fn from_html_table_with_colgroup() {
    // Test table with colgroup (covers L232-236 _ => {} branch)
    let html = "<table><colgroup><col span=\"2\"></colgroup><tr><td>A</td><td>B</td></tr></table>";
    let (stdout, _) = TvaCmd::new()
        .args(&["from", "html", "--table"])
        .stdin(html)
        .run();
    assert_eq!(normalize_newlines(&stdout), "A\tB\n");
}

#[test]
fn from_html_table_row_with_comment() {
    // Test row with comment node (covers L258-259 _ => {} branch)
    let html = "<table><tr><td>A</td><!-- comment --><td>B</td></tr></table>";
    let (stdout, _) = TvaCmd::new()
        .args(&["from", "html", "--table"])
        .stdin(html)
        .run();
    assert_eq!(normalize_newlines(&stdout), "A\tB\n");
}

#[test]
fn from_html_col_attr_paren_invalid_syntax() {
    // Test invalid attr() syntax without closing paren (covers L358-359)
    let (_, stderr) = TvaCmd::new()
        .args(&[
            "from",
            "html",
            "--row",
            "div",
            "--col",
            "Name:attr(href", // Missing closing )
            "tests/data/from_html/basic.html",
        ])
        .run_fail();
    assert!(stderr.contains("Invalid attr() syntax"));
}
