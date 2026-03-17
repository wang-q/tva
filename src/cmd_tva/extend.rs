use crate::cmd_tva::expr;
use crate::libs::cli::expr_common_args;
use clap::*;

/// Creates an extend subcommand that acts as an alias for `expr -m extend`
pub fn make_subcommand() -> Command {
    Command::new("extend")
        .about("Adds new columns to each row")
        .after_help(
            r###"This is an alias for 'tva expr -m extend'.
The expression result is appended to the original row as new column(s).
"###,
        )
        .args(expr_common_args())
}

pub fn execute(matches: &ArgMatches) -> anyhow::Result<()> {
    expr::execute_with_mode(matches, "extend")
}
