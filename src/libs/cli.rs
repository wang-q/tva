//! Header-related CLI argument handling utilities.

use crate::libs::tsv::header::HeaderConfig;
use clap::{Arg, ArgAction};

/// Full conventions document included at compile time.
const CONVENTIONS_FULL: &str = include_str!("../../docs/conventions.md");

/// Help text for field selection syntax, extracted from docs/conventions.md.
pub static FIELD_SYNTAX_HELP: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| {
        extract_markdown_section(CONVENTIONS_FULL, "Field Selection Syntax")
    });

/// Help text for header handling, extracted from docs/conventions.md.
pub static HEADER_HELP: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    extract_markdown_section(CONVENTIONS_FULL, "Header Handling")
});

/// Help text for expression syntax, extracted from docs/conventions.md.
pub static EXPR_SYNTAX_HELP: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| {
        extract_markdown_section(CONVENTIONS_FULL, "Expr Syntax")
    });

/// Extracts a section from a markdown document by its header.
///
/// Finds the section starting with `## {section_name}` and returns everything
/// from that header up to (but not including) the next `## ` header.
pub fn extract_markdown_section(content: &str, section_name: &str) -> String {
    let needle = format!("## {}", section_name);

    // Find the section start
    let start = match content.find(&needle) {
        Some(pos) => pos,
        None => return format!("# {}\n\nDocumentation not found.\n", section_name),
    };

    // Find the end of the section (start of next ## section)
    let rest = &content[start + needle.len()..];
    let end_offset = rest.find("\n## ").unwrap_or(rest.len());

    // Extract the section
    content[start..start + needle.len() + end_offset].to_string()
}

/// Returns the standard header-related CLI arguments.
///
/// This includes all four header detection modes:
/// - `--header` / `-H`: FirstLine mode
/// - `--header-lines N`: LinesN mode
/// - `--header-hash`: HashLines mode
/// - `--header-hash1`: HashLines1 mode
pub fn header_args() -> Vec<Arg> {
    vec![
        Arg::new("header")
            .long("header")
            .short('H')
            .action(ArgAction::SetTrue)
            .help("Treat the first line as header (FirstLine mode)"),
        Arg::new("header-lines")
            .long("header-lines")
            .num_args(1)
            .value_parser(clap::value_parser!(usize))
            .help("Treat exactly N lines as header (LinesN mode, including empty lines)"),
        Arg::new("header-hash")
            .long("header-hash")
            .action(ArgAction::SetTrue)
            .help("Treat consecutive '#' lines as header (HashLines mode)"),
        Arg::new("header-hash1")
            .long("header-hash1")
            .action(ArgAction::SetTrue)
            .help("Treat '#' lines plus next line as header (HashLines1 mode, for column names)"),
    ]
}

/// Returns header args for commands that need column names (like select).
///
/// Only includes modes that provide column names:
/// - `--header` / `-H`: FirstLine mode (first line contains column names)
/// - `--header-hash1`: HashLines1 mode (hash comments + next line with column names)
pub fn header_args_with_columns() -> Vec<Arg> {
    vec![
        Arg::new("header")
            .long("header")
            .short('H')
            .action(ArgAction::SetTrue)
            .help("Treat the first line as header (FirstLine mode)"),
        Arg::new("header-hash1")
            .long("header-hash1")
            .action(ArgAction::SetTrue)
            .help("Treat '#' lines plus next line as header (HashLines1 mode)"),
    ]
}

/// Returns only the basic `--header` argument.
///
/// Use this for commands that only need simple header support.
pub fn header_arg_basic() -> Arg {
    Arg::new("header")
        .long("header")
        .short('H')
        .action(ArgAction::SetTrue)
        .help("Treat the first line as header (FirstLine mode)")
}

/// Builds a `HeaderConfig` from parsed command-line arguments.
///
/// # Arguments
/// * `matches` - The parsed argument matches from clap
/// * `exit_on_error` - If true, calls `std::process::exit(1)` on error;
///   otherwise returns an error message
///
/// # Errors
/// Returns an error message string if conflicting header options are provided.
pub fn build_header_config(
    matches: &clap::ArgMatches,
    exit_on_error: bool,
) -> Result<HeaderConfig, String> {
    let has_header_flag = matches.get_flag("header");

    // Safely get optional arguments that may not be defined for all commands
    // by checking if the argument ID exists before accessing
    let arg_ids: std::collections::HashSet<String> =
        matches.ids().map(|id| id.as_str().to_string()).collect();

    let header_lines: Option<usize> = if arg_ids.contains("header-lines") {
        matches.get_one::<usize>("header-lines").copied()
    } else {
        None
    };

    let header_hash = if arg_ids.contains("header-hash") {
        matches.get_flag("header-hash")
    } else {
        false
    };

    let header_hash1 = if arg_ids.contains("header-hash1") {
        matches.get_flag("header-hash1")
    } else {
        false
    };

    // Count how many header modes are specified
    let mode_count = [
        has_header_flag,
        header_lines.is_some(),
        header_hash,
        header_hash1,
    ]
    .iter()
    .filter(|&&x| x)
    .count();

    if mode_count > 1 {
        let msg =
            "only one of --header, --header-lines, --header-hash, --header-hash1 can be specified";
        if exit_on_error {
            eprintln!("tva: {}", msg);
            std::process::exit(1);
        }
        return Err(msg.to_string());
    }

    let mut config = HeaderConfig::new();

    if has_header_flag {
        config = config.enabled().first_line();
    } else if let Some(n) = header_lines {
        if n == 0 {
            let msg = "--header-lines requires a positive number";
            if exit_on_error {
                eprintln!("tva: {}", msg);
                std::process::exit(1);
            }
            return Err(msg.to_string());
        }
        config = config.enabled().lines_n(n);
    } else if header_hash {
        config = config.enabled().hash_lines();
    } else if header_hash1 {
        config = config.enabled().hash_lines1();
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Arg, Command};

    fn create_test_command() -> Command {
        Command::new("test")
            .args(header_args())
            .arg(Arg::new("other").long("other"))
    }

    #[test]
    fn test_header_config_default() {
        let cmd = create_test_command();
        let matches = cmd.try_get_matches_from(["test"]).unwrap();
        let config = build_header_config(&matches, false).unwrap();

        assert!(!config.enabled);
        assert!(matches!(
            config.mode,
            crate::libs::tsv::header::HeaderMode::FirstLine
        ));
    }

    #[test]
    fn test_header_config_firstline() {
        let cmd = create_test_command();
        let matches = cmd.try_get_matches_from(["test", "--header"]).unwrap();
        let config = build_header_config(&matches, false).unwrap();

        assert!(config.enabled);
        assert!(matches!(
            config.mode,
            crate::libs::tsv::header::HeaderMode::FirstLine
        ));
    }

    #[test]
    fn test_header_config_firstline_short() {
        let cmd = create_test_command();
        let matches = cmd.try_get_matches_from(["test", "-H"]).unwrap();
        let config = build_header_config(&matches, false).unwrap();

        assert!(config.enabled);
        assert!(matches!(
            config.mode,
            crate::libs::tsv::header::HeaderMode::FirstLine
        ));
    }

    #[test]
    fn test_header_config_lines_n() {
        let cmd = create_test_command();
        let matches = cmd
            .try_get_matches_from(["test", "--header-lines", "3"])
            .unwrap();
        let config = build_header_config(&matches, false).unwrap();

        assert!(config.enabled);
        assert!(matches!(
            config.mode,
            crate::libs::tsv::header::HeaderMode::LinesN(3)
        ));
    }

    #[test]
    fn test_header_config_hash_lines() {
        let cmd = create_test_command();
        let matches = cmd.try_get_matches_from(["test", "--header-hash"]).unwrap();
        let config = build_header_config(&matches, false).unwrap();

        assert!(config.enabled);
        assert!(matches!(
            config.mode,
            crate::libs::tsv::header::HeaderMode::HashLines
        ));
    }

    #[test]
    fn test_header_config_hash_lines1() {
        let cmd = create_test_command();
        let matches = cmd
            .try_get_matches_from(["test", "--header-hash1"])
            .unwrap();
        let config = build_header_config(&matches, false).unwrap();

        assert!(config.enabled);
        assert!(matches!(
            config.mode,
            crate::libs::tsv::header::HeaderMode::HashLines1
        ));
    }

    #[test]
    fn test_header_config_conflicting_options_header_and_hash1() {
        // Test --header and --header-hash1 conflict
        let cmd = create_test_command();
        let matches = cmd
            .try_get_matches_from(["test", "--header", "--header-hash1"])
            .unwrap();
        let result = build_header_config(&matches, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("only one of"));
    }

    #[test]
    fn test_header_config_conflicting_options_hash_and_hash1() {
        // Test --header-hash and --header-hash1 conflict
        let cmd = create_test_command();
        let matches = cmd
            .try_get_matches_from(["test", "--header-hash", "--header-hash1"])
            .unwrap();
        let result = build_header_config(&matches, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_header_config_conflicting_options_header_and_lines() {
        // Test --header and --header-lines conflict
        let cmd = create_test_command();
        let matches = cmd
            .try_get_matches_from(["test", "--header", "--header-lines", "2"])
            .unwrap();
        let result = build_header_config(&matches, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_header_config_lines_n_zero_error() {
        let cmd = create_test_command();
        let matches = cmd
            .try_get_matches_from(["test", "--header-lines", "0"])
            .unwrap();
        let result = build_header_config(&matches, false);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("positive number"));
    }

    #[test]
    fn test_header_args_with_columns() {
        let args = header_args_with_columns();

        // Should have exactly 2 args
        assert_eq!(args.len(), 2);

        // Check --header is present
        let header_arg = args.iter().find(|a| a.get_id().as_str() == "header");
        assert!(header_arg.is_some());

        // Check --header-hash1 is present
        let hash1_arg = args.iter().find(|a| a.get_id().as_str() == "header-hash1");
        assert!(hash1_arg.is_some());

        // --header-lines and --header-hash should NOT be present
        let lines_arg = args.iter().find(|a| a.get_id().as_str() == "header-lines");
        assert!(lines_arg.is_none());

        let hash_arg = args.iter().find(|a| a.get_id().as_str() == "header-hash");
        assert!(hash_arg.is_none());
    }

    #[test]
    fn test_header_arg_basic() {
        let arg = header_arg_basic();

        assert_eq!(arg.get_id().as_str(), "header");
        assert!(arg.get_short() == Some('H'));
    }

    #[test]
    fn test_extract_markdown_section_found() {
        let content = "## Section A\nContent A\n## Section B\nContent B";
        let result = extract_markdown_section(content, "Section A");
        assert!(result.contains("## Section A"));
        assert!(result.contains("Content A"));
        assert!(!result.contains("Section B"));
    }

    #[test]
    fn test_extract_markdown_section_not_found() {
        let content = "## Section A\nContent A";
        let result = extract_markdown_section(content, "NonExistent");
        assert_eq!(result, "# NonExistent\n\nDocumentation not found.\n");
    }

    #[test]
    fn test_extract_markdown_section_last_section() {
        // Section at the end with no following section
        let content = "## Section A\nContent A\n## Section B\nContent B";
        let result = extract_markdown_section(content, "Section B");
        assert!(result.contains("## Section B"));
        assert!(result.contains("Content B"));
        assert!(!result.contains("Section A"));
    }
}
