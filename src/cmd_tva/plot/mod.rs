pub mod bin2d;
pub mod r#box;
pub mod point;

use clap::{ArgMatches, Command};

pub fn make_subcommand() -> Command {
    Command::new("plot")
        .about("Plotting commands")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(point::make_subcommand())
        .subcommand(r#box::make_subcommand())
        .subcommand(bin2d::make_subcommand())
}

pub fn execute(matches: &ArgMatches) -> anyhow::Result<()> {
    match matches.subcommand() {
        Some(("point", sub_matches)) => point::execute(sub_matches),
        Some(("box", sub_matches)) => r#box::execute(sub_matches),
        Some(("bin2d", sub_matches)) => bin2d::execute(sub_matches),
        _ => unreachable!(),
    }
}
