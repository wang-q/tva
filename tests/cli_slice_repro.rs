#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn slice_multi_file_header_duplication() {
    let file1_content = "Header\n1\n2\n";
    let file2_content = "Header\n3\n4\n";

    let temp_dir = tempfile::tempdir().unwrap();
    let file1_path = temp_dir.path().join("file1.tsv");
    let file2_path = temp_dir.path().join("file2.tsv");

    std::fs::write(&file1_path, file1_content).unwrap();
    std::fs::write(&file2_path, file2_content).unwrap();

    let (stdout, _) = TvaCmd::new()
        .args(&[
            "slice",
            "-H",
            "-r",
            "1-2", // Keep 1st data row (header is separate) -> effectively rows 1-2 + header?
            // slice logic: -H keeps line 1. -r 1-2 keeps line 1 and 2.
            // So effectively keeps line 1 (header), line 1 (data), line 2 (data).
            file1_path.to_str().unwrap(),
            file2_path.to_str().unwrap(),
        ])
        .run();

    // Current behavior expected (based on code analysis):
    // File 1: Header, 1, 2
    // File 2: Header, 3, 4
    // Output: Header\n1\n2\nHeader\n3\n4\n

    // Output: Header\n1\n2\n3\n4\n (Header from file 2 skipped)
    assert_eq!(
        stdout.matches("Header").count(),
        1,
        "Header should appear exactly once"
    );
    // With -r 1-2, it keeps line 1 and 2 of each file.
    // File 1: Header (L1 - skip logic prints it), 1 (L2 - keep), 2 (L3 - keep) -> No, line_num logic includes header as L1.
    // Wait, line_num counts physical lines.
    // File 1:
    // L1: Header. Keep (due to -H and -r 1).
    // L2: 1. Keep (due to -r 2).
    // L3: 2. Drop.
    // File 2:
    // L1: Header. Skip printing (header_written=true). But return Ok(()). So NOT printed.
    // L2: 3. Keep (due to -r 2).
    // L3: 4. Drop.

    // Result: Header\n1\n3\n
    // My previous assertion expected 1\n2\n3\n4\n which implies keeping 2 lines of data.
    // -r 1-2 means L1 and L2.
    // L1 is Header. L2 is Data Row 1.
    // So we keep Header and Data Row 1 from each file.
    // File 1: Header, 1.
    // File 2: Header (skipped), 3.
    // Output: Header\n1\n3\n.

    assert!(
        stdout.contains("1\n3\n"),
        "Should contain first data row of each file"
    );
    assert!(
        !stdout.contains("2\n"),
        "Should not contain second data row (L3)"
    );
    assert!(
        !stdout.contains("4\n"),
        "Should not contain second data row (L3)"
    );
}
