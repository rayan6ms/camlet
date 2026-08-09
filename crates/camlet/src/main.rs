#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::process::ExitCode;

use camlet::{Cli, CliAction};

fn main() -> ExitCode {
    let arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
    let cli = match Cli::parse(arguments.iter().cloned()) {
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
        CliAction::Run => {
            if let Some(exit_code) = relaunch_with_x11_if_needed(&arguments) {
                return exit_code;
            }
            match camlet::run(&cli) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("camlet: application failed: {error}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn relaunch_with_x11_if_needed(arguments: &[std::ffi::OsString]) -> Option<ExitCode> {
    let requested_backend = std::env::var("WINIT_UNIX_BACKEND").ok();
    let wayland_available = std::env::var_os("WAYLAND_DISPLAY").is_some()
        || std::env::var_os("WAYLAND_SOCKET").is_some();
    if requested_backend.as_deref() == Some("wayland")
        || std::env::var_os("CAMLET_X11_RELAUNCHED").is_some()
        || std::env::var_os("DISPLAY").is_none()
        || !wayland_available
    {
        return None;
    }

    let executable = std::env::current_exe().ok()?;
    match std::process::Command::new(executable)
        .args(arguments)
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("WAYLAND_SOCKET")
        .env("WINIT_UNIX_BACKEND", "x11")
        .env("CAMLET_X11_RELAUNCHED", "1")
        .status()
    {
        Ok(status) if status.success() => Some(ExitCode::SUCCESS),
        Ok(status) => Some(
            status
                .code()
                .and_then(|code| u8::try_from(code).ok())
                .map_or(ExitCode::FAILURE, ExitCode::from),
        ),
        Err(error) => {
            eprintln!("camlet: could not relaunch with X11 positioning support: {error}");
            None
        }
    }
}

#[cfg(not(target_os = "linux"))]
const fn relaunch_with_x11_if_needed(_arguments: &[std::ffi::OsString]) -> Option<ExitCode> {
    None
}
