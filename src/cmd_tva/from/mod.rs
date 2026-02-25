pub mod csv;
pub mod xlsx;

use clap::{ArgMatches, Command};

pub fn make_subcommand() -> Command {
    Command::new("from")
        .about("Convert from other formats to TSV")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(csv::make_subcommand())
        .subcommand(xlsx::make_subcommand())
}

pub fn execute(matches: &ArgMatches) -> anyhow::Result<()> {
    match matches.subcommand() {
        Some(("csv", sub_matches)) => csv::execute(sub_matches),
        Some(("xlsx", sub_matches)) => xlsx::execute(sub_matches),
        _ => unreachable!(),
    }
}
