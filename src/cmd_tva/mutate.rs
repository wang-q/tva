use crate::cmd_tva::expr;
use crate::libs::cli::expr_common_args;
use clap::*;

/// Creates a mutate subcommand that acts as an alias for `expr -m mutate`
pub fn make_subcommand() -> Command {
    Command::new("mutate")
        .about("Modifies an existing column in place")
        .after_help(
            r###"This is an alias for 'tva expr -m mutate'.
The expression must include an 'as @column' binding to specify which column to modify.
"###,
        )
        .args(expr_common_args())
}

pub fn execute(matches: &ArgMatches) -> anyhow::Result<()> {
    expr::execute_with_mode(matches, "mutate")
}
