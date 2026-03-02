#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn pup_compat_basic_selectors() {
    let index_html = "tests/data/pup_tests/index.html";

    let (stdout, _) = TvaCmd::new()
        .args(&["from", "html", "-q", "title", index_html])
        .run();
    assert!(stdout.contains("Go (programming language)"));

    let (stdout, _) = TvaCmd::new()
        .args(&["from", "html", "-q", "#footer", index_html])
        .run();
    assert!(stdout.contains("footer-info"));

    let (stdout, _) = TvaCmd::new()
        .args(&["from", "html", "-q", ".catlinks", index_html])
        .run();
    assert!(stdout.contains("mw-normal-catlinks"));
}

#[test]
fn pup_compat_combinators() {
    let index_html = "tests/data/pup_tests/index.html";

    let (stdout, _) = TvaCmd::new()
        .args(&["from", "html", "-q", "table li", index_html])
        .run();
    assert!(stdout.contains("<li>"));

    let (stdout, _) = TvaCmd::new()
        .args(&["from", "html", "-q", "ul > li", index_html])
        .run();
    assert!(stdout.contains("<li>"));
}

#[test]
fn pup_compat_attributes() {
    let index_html = "tests/data/pup_tests/index.html";

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "from",
            "html",
            "-q",
            "table a[title=\"The Practice of Programming\"]",
            index_html,
        ])
        .run();
    assert!(stdout.contains("The Practice of Programming"));

    let (stdout, _) = TvaCmd::new()
        .args(&["from", "html", "-q", "a[rel]", index_html])
        .run();
    assert!(stdout.contains("rel=\""));
}

#[test]
fn pup_compat_pseudo_classes() {
    let index_html = "tests/data/pup_tests/index.html";

    let (stdout, _) = TvaCmd::new()
        .args(&["from", "html", "-q", "table li:first-child", index_html])
        .run();
    assert!(!stdout.is_empty());

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "from",
            "html",
            "-q",
            ".navbox-list li:nth-child(1)",
            index_html,
        ])
        .run();
    assert!(!stdout.is_empty());

    let (stdout, _) = TvaCmd::new()
        .args(&["from", "html", "-q", ":empty", index_html])
        .run();
    assert!(!stdout.is_empty());
}

#[test]
fn pup_compat_display_functions() {
    let index_html = "tests/data/pup_tests/index.html";

    let (stdout, _) = TvaCmd::new()
        .args(&["from", "html", "-q", "h1#firstHeading text{}", index_html])
        .run();
    assert!(stdout.contains("Go (programming language)"));
    assert!(!stdout.contains("<h1"));

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "from",
            "html",
            "-q",
            "link[rel=\"canonical\"] attr{href}",
            index_html,
        ])
        .run();
    assert!(stdout.contains("http://en.wikipedia.org/wiki/Go_(programming_language)"));
}
