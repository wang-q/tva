pub mod point;

use clap::{ArgMatches, Command};

pub fn make_subcommand() -> Command {
    Command::new("plot")
        .about("Plotting commands")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(point::make_subcommand())
}

pub fn execute(matches: &ArgMatches) -> anyhow::Result<()> {
    match matches.subcommand() {
        Some(("point", sub_matches)) => point::execute(sub_matches),
        _ => unreachable!(),
    }
}
