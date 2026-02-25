pub mod csv;

use clap::{ArgMatches, Command};

pub fn make_subcommand() -> Command {
    Command::new("from")
        .about("Convert from other formats to TSV")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(csv::make_subcommand())
}

pub fn execute(matches: &ArgMatches) -> anyhow::Result<()> {
    match matches.subcommand() {
        Some(("csv", sub_matches)) => csv::execute(sub_matches),
        _ => unreachable!(),
    }
}
