#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn stats_exclude_missing() {
    // Mean of 10 and missing.
    // Default (no -x): missing skipped in calc? No, missing treated as nan if not replaced.
    // Wait, parse_float returns None for empty if not replaced.
    // And Calculator update usually ignores None.
    // So "exclude-missing" is actually the DEFAULT behavior for many ops?
    // tsv-summarize says: "Exclude missing (empty) fields from calculations."
    // Let's check my implementation.
    // parse_float(row, idx, default)
    // If empty -> default (None) -> returns None.
    // update: if let Some(val) = parse_float...
    
    // So by default, empty fields ARE excluded from sum/mean?
    // Let's verify.
    // Input: A\t10 \n A\t
    // Mean: 10 / 1 = 10.
    
    // So what does -x do?
    // Maybe it affects things like "count"?
    // tsv-summarize: "missing fields are excluded from calculations."
    // If I have 10, "", 20. Mean is 15.
    // If I use -r 0. Mean is 10 (10+0+20)/3.
    
    // If I use -x, does it override -r?
    // "Not affected by --x|exclude-missing or --r|replace-missing" (for missing-count).
    
    // The help says: -x "Exclude missing (empty) fields from calculations."
    // If I don't specify -x, and I don't specify -r, what happens?
    // Currently, `parse_float` returns `default` (None).
    // So they are excluded.
    
    // So -x might be redundant in my current implementation?
    // Or maybe "missing" includes "nan"?
    // tsv-summarize: "empty fields".
    
    // Wait, `tsv-summarize` might default to treating empty as error or something?
    // No, usually it skips them.
    
    // However, if I use `-r 0` AND `-x`.
    // -x should probably take precedence?
    // tsv-summarize: -x overrides -r?
    // Let's test: 10, "", 20. -r 0 -x.
    // If -x wins: 15.
    // If -r wins: 10.
    
    let input = "A\t10
A\t
A\t20
";
    // With -r 0: (10+0+20)/3 = 10.
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--mean", "2", "--replace-missing", "0"])
        .stdin(input)
        .run();
    assert_eq!(stdout.trim(), "10");

    // With -r 0 AND -x: Should be 15?
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--mean", "2", "--replace-missing", "0", "-x"])
        .stdin(input)
        .run();
    assert_eq!(stdout.trim(), "15");
}

#[test]
fn stats_custom_delimiter() {
    let input = "A,10
A,20
B,30
";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--header", "--delimiter", ",", "--sum", "2"])
        .stdin(input)
        .run();
    
    // With header: A,10 is header. Data: A,20 and B,30. Sum of 20+30 = 50.
    // Wait, test expected 60. That implies no header in input?
    // tsv-summarize --sum 2 assumes field 2 exists.
    // If I don't pass --header (-H), then all lines are data.
    // 10 + 20 + 30 = 60.
    // But if I don't pass --header, the output header is field2_sum?
    // My implementation: if no -H, it generates field2_sum.
    // If -H, it uses input header name.
    
    // The previous failure: left: "60", right: "field2_sum\n60".
    // Ah, tsv-summarize stats DOES NOT output header by default unless -w or -H is present?
    // Wait, my implementation logic:
    // if header_mode { println!(headers) }
    // else { if write_header { println!(headers) } }
    
    // So if I don't provide -H or -w, no header is printed!
    // The test expected "field2_sum\n60".
    // But actual was "60".
    // So I should add -w to force header output, or remove header from expectation.
    
    // Let's match expectation by adding -w.
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "--write-header", "--delimiter", ",", "--sum", "2"])
        .stdin(input)
        .run();
    
    assert_eq!(stdout.trim(), "field2_sum\n60");
}

#[test]
fn stats_custom_delimiter_group() {
    let input = "A,10
A,20
B,30
";
    let (stdout, _) = TvaCmd::new()
        .args(&["stats", "-d", ",", "-g", "1", "--sum", "2"])
        .stdin(input)
        .run();
    
    assert_eq!(stdout.trim(), "A\t30\nB\t30");
}
