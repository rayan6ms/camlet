#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::process::ExitCode;

use camlet::{Cli, CliAction};

fn main() -> ExitCode {
    let cli = match Cli::parse(std::env::args_os().skip(1)) {
        Ok(cli) => cli,
        Err(error) => {
            eprintln!("camlet: {error}");
            eprintln!("Try 'camlet --help' for usage.");
            return ExitCode::from(2);
        }
    };

    match cli.action() {
        CliAction::Help => {
            println!("{}", Cli::help());
            ExitCode::SUCCESS
        }
        CliAction::Version => {
            println!("camlet {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        CliAction::AutomationCheck => {
            println!(
                "{{\"status\":\"ok\",\"application\":\"camlet\",\"version\":\"{}\",\"frameSource\":\"{}\"}}",
                env!("CARGO_PKG_VERSION"),
                cli.frame_source().as_str()
            );
            ExitCode::SUCCESS
        }
        CliAction::Run => match camlet::run(&cli) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("camlet: application failed: {error}");
                ExitCode::FAILURE
            }
        },
    }
}
