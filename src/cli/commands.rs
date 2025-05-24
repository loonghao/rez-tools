use crate::cli::CliApp;
use clap::{Arg, Command};

/// Build the main command with global options
pub fn build_main_command() -> Command {
    Command::new("rt")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("A suite tool command line for rez")
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose logging")
                .action(clap::ArgAction::SetTrue)
                .global(true),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("Suppress output")
                .action(clap::ArgAction::SetTrue)
                .global(true),
        )
        .subcommand(
            Command::new("list")
                .about("List all available plugins")
                .alias("ls"),
        )
}

/// Handle the list subcommand
pub fn handle_list_command(app: &CliApp) -> crate::error::Result<i32> {
    app.list_plugins();
    Ok(0)
}

/// Setup logging based on command line arguments
pub fn setup_logging(verbose: bool, quiet: bool) {
    use env_logger::Env;

    let env = Env::default().filter_or(
        "RUST_LOG",
        if quiet {
            "error"
        } else if verbose {
            "debug"
        } else {
            "info"
        },
    );

    env_logger::init_from_env(env);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_main_command() {
        let cmd = build_main_command();
        assert_eq!(cmd.get_name(), "rt");
        assert!(cmd.get_subcommands().any(|s| s.get_name() == "list"));
    }
}
