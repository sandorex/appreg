use std::process::ExitCode;
use anyhow::Error;

mod cli;
mod installer;

fn main() -> ExitCode {
    use clap::Parser;
    let cli_args = cli::Cli::parse();

    let result = match cli_args.cmd {
        cli::CliCommands::Update => todo!(),
        cli::CliCommands::Install(x) => installer::cmd_install(cli_args.dry_run, x),
    };

    match result {
        Ok(x) => ExitCode::from(x),
        Err(err) => {
            eprintln!("Error: {}", err);
            ExitCode::from(1)
        }
    }
}
