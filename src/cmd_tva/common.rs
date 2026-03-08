//! Common utilities shared across tva commands.

use crate::libs::tsv::header::HeaderConfig;
use clap::{Arg, ArgAction};

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
            .help("Treat the first non-empty line as header (FirstLine mode)"),
        Arg::new("header-lines")
            .long("header-lines")
            .num_args(1)
            .value_parser(clap::value_parser!(usize))
            .help("Treat exactly N non-empty lines as header (LinesN mode)"),
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
            .help("Treat the first non-empty line as header (contains column names)"),
        Arg::new("header-hash1")
            .long("header-hash1")
            .action(ArgAction::SetTrue)
            .help("Treat '#' lines plus next line as header (for column names)"),
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
        .help("Treat the first non-empty line as header")
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
