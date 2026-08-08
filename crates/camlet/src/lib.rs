#![doc = "Native Iced application shell for Camlet."]

mod app;
mod automation;
mod cli;
mod gpu_overlay;
mod profile;
mod renderer;
mod screenshot;

pub use cli::{AutomationMode, Cli, CliAction, CliError};

/// Native application launch failure.
#[derive(Debug, thiserror::Error)]
pub enum RunError {
    /// A packaged automation fixture was invalid or inaccessible.
    #[error(transparent)]
    Automation(#[from] automation::AutomationError),
    /// Iced could not initialize or run the native shell.
    #[error(transparent)]
    Iced(#[from] iced::Error),
}

/// Runs the command represented by parsed command-line arguments.
///
/// # Errors
///
/// Returns an Iced error if window or renderer initialization fails.
pub fn run(cli: &Cli) -> Result<(), RunError> {
    app::run(cli)
}
