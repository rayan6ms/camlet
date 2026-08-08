use std::ffi::OsString;
use std::fmt;
use std::path::{Path, PathBuf};

use camlet_core::FrameSourceKind;

/// Optional unattended behavior used by packaged smoke tests.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AutomationMode {
    /// Run until a user or operating-system close request.
    #[default]
    None,
    /// Close automatically after the Iced event loop has started.
    ExitAfterLaunch,
    /// Run a validated unattended scenario and close on its final action.
    Scripted,
}

/// High-level command selected by command-line arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliAction {
    /// Start the Iced application.
    Run,
    /// Print help and exit.
    Help,
    /// Print the version and exit.
    Version,
    /// Validate the binary and configuration without opening a window.
    AutomationCheck,
}

/// Validated startup configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cli {
    action: CliAction,
    automation: AutomationMode,
    frame_source: FrameSourceKind,
    screenshot: Option<PathBuf>,
    profile_directory: Option<PathBuf>,
    automation_script: Option<PathBuf>,
    automation_output: Option<PathBuf>,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            action: CliAction::Run,
            automation: AutomationMode::None,
            frame_source: FrameSourceKind::Real,
            screenshot: None,
            profile_directory: None,
            automation_script: None,
            automation_output: None,
        }
    }
}

impl Cli {
    /// Parses command-line arguments without reading process-global state.
    ///
    /// # Errors
    ///
    /// Returns a typed error for unknown, duplicated, incomplete, or non-Unicode arguments.
    pub fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<Self, CliError> {
        let mut cli = Self::default();
        let mut arguments = arguments.into_iter();
        let mut source_was_set = false;
        let mut screenshot_was_set = false;
        let mut profile_was_set = false;
        let mut automation_script_was_set = false;
        let mut automation_output_was_set = false;

        while let Some(argument) = arguments.next() {
            let argument = argument.into_string().map_err(CliError::NonUnicode)?;

            match argument.as_str() {
                "--help" | "-h" => cli.set_action(CliAction::Help)?,
                "--version" | "-V" => cli.set_action(CliAction::Version)?,
                "--automation-check" => cli.set_action(CliAction::AutomationCheck)?,
                "--automation-exit" => {
                    if cli.automation != AutomationMode::None {
                        return Err(CliError::DuplicateOption("--automation-exit"));
                    }
                    cli.automation = AutomationMode::ExitAfterLaunch;
                }
                "--automation-script" => {
                    if automation_script_was_set || cli.automation != AutomationMode::None {
                        return Err(CliError::DuplicateOption("--automation-script"));
                    }
                    cli.automation_script = Some(next_path(&mut arguments, "--automation-script")?);
                    cli.automation = AutomationMode::Scripted;
                    automation_script_was_set = true;
                }
                "--automation-output" => {
                    if automation_output_was_set {
                        return Err(CliError::DuplicateOption("--automation-output"));
                    }
                    cli.automation_output = Some(next_path(&mut arguments, "--automation-output")?);
                    automation_output_was_set = true;
                }
                "--frame-source" => {
                    if source_was_set {
                        return Err(CliError::DuplicateOption("--frame-source"));
                    }
                    let value = arguments
                        .next()
                        .ok_or(CliError::MissingValue("--frame-source"))?
                        .into_string()
                        .map_err(CliError::NonUnicode)?;
                    cli.frame_source = FrameSourceKind::parse(&value)
                        .ok_or(CliError::InvalidFrameSource(value))?;
                    source_was_set = true;
                }
                "--screenshot" => {
                    if screenshot_was_set {
                        return Err(CliError::DuplicateOption("--screenshot"));
                    }
                    let value = arguments
                        .next()
                        .ok_or(CliError::MissingValue("--screenshot"))?;
                    if value.is_empty() {
                        return Err(CliError::MissingValue("--screenshot"));
                    }
                    cli.screenshot = Some(PathBuf::from(value));
                    screenshot_was_set = true;
                }
                "--profile-dir" => {
                    if profile_was_set {
                        return Err(CliError::DuplicateOption("--profile-dir"));
                    }
                    let value = arguments
                        .next()
                        .ok_or(CliError::MissingValue("--profile-dir"))?;
                    if value.is_empty() {
                        return Err(CliError::MissingValue("--profile-dir"));
                    }
                    cli.profile_directory = Some(PathBuf::from(value));
                    profile_was_set = true;
                }
                _ => return Err(CliError::UnknownArgument(argument)),
            }
        }

        if cli.action != CliAction::Run
            && (cli.automation != AutomationMode::None || cli.screenshot.is_some())
        {
            return Err(CliError::ConflictingOptions);
        }
        if cli.automation_script.is_some() != cli.automation_output.is_some()
            || (cli.automation == AutomationMode::Scripted && cli.screenshot.is_some())
        {
            return Err(CliError::ConflictingOptions);
        }

        Ok(cli)
    }

    /// Returns static usage text.
    #[must_use]
    pub const fn help() -> &'static str {
        "Camlet native camera overlay\n\nUsage: camlet [OPTIONS]\n\nOptions:\n  --frame-source <real|synthetic>  Select the frame producer\n  --profile-dir <directory>       Override native settings directory\n  --screenshot <path.ppm>         Capture the native window over white and exit\n  --automation-check              Validate and exit without a window\n  --automation-exit               Open the window, then close automatically\n  --automation-script <file>      Run a validated unattended scenario\n  --automation-output <directory> Write scenario readiness/results here\n  -h, --help                      Print help\n  -V, --version                   Print version"
    }

    /// Returns the requested top-level action.
    #[must_use]
    pub const fn action(&self) -> CliAction {
        self.action
    }

    /// Returns the selected automation behavior.
    #[must_use]
    pub const fn automation(&self) -> AutomationMode {
        self.automation
    }

    /// Returns the selected frame source.
    #[must_use]
    pub const fn frame_source(&self) -> FrameSourceKind {
        self.frame_source
    }

    /// Returns the optional unattended screenshot destination.
    #[must_use]
    pub fn screenshot(&self) -> Option<&Path> {
        self.screenshot.as_deref()
    }

    /// Returns the optional settings directory override used by tests and portable runs.
    #[must_use]
    pub fn profile_directory(&self) -> Option<&Path> {
        self.profile_directory.as_deref()
    }

    /// Returns the optional packaged-style automation fixture.
    #[must_use]
    pub fn automation_script(&self) -> Option<&Path> {
        self.automation_script.as_deref()
    }

    /// Returns the isolated automation artifact directory.
    #[must_use]
    pub fn automation_output(&self) -> Option<&Path> {
        self.automation_output.as_deref()
    }

    fn set_action(&mut self, action: CliAction) -> Result<(), CliError> {
        if self.action != CliAction::Run {
            return Err(CliError::ConflictingOptions);
        }
        self.action = action;
        Ok(())
    }
}

fn next_path(
    arguments: &mut impl Iterator<Item = OsString>,
    option: &'static str,
) -> Result<PathBuf, CliError> {
    let value = arguments.next().ok_or(CliError::MissingValue(option))?;
    if value.is_empty() {
        Err(CliError::MissingValue(option))
    } else {
        Ok(PathBuf::from(value))
    }
}

/// Command-line validation failure.
#[derive(Debug, PartialEq, Eq)]
pub enum CliError {
    /// An argument was not valid Unicode.
    NonUnicode(OsString),
    /// An unsupported argument was passed.
    UnknownArgument(String),
    /// An option requiring a value was last.
    MissingValue(&'static str),
    /// A singleton option was repeated.
    DuplicateOption(&'static str),
    /// The frame-source value was not supported.
    InvalidFrameSource(String),
    /// Mutually exclusive actions were combined.
    ConflictingOptions,
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonUnicode(_) => formatter.write_str("arguments must be valid Unicode"),
            Self::UnknownArgument(argument) => write!(formatter, "unknown argument '{argument}'"),
            Self::MissingValue(option) => write!(formatter, "missing value for {option}"),
            Self::DuplicateOption(option) => write!(formatter, "{option} may only be used once"),
            Self::InvalidFrameSource(source) => {
                write!(
                    formatter,
                    "invalid frame source '{source}'; expected real or synthetic"
                )
            }
            Self::ConflictingOptions => formatter.write_str("conflicting command options"),
        }
    }
}

impl std::error::Error for CliError {}

#[cfg(test)]
mod tests {
    use super::{AutomationMode, Cli, CliAction, CliError};
    use camlet_core::FrameSourceKind;

    fn arguments<'a>(values: &'a [&'a str]) -> impl Iterator<Item = std::ffi::OsString> + 'a {
        values.iter().map(std::ffi::OsString::from)
    }

    #[test]
    fn defaults_to_real_interactive_run() {
        let cli = Cli::parse(arguments(&[])).unwrap_or_else(|error| unreachable!("{error}"));

        assert_eq!(cli.action(), CliAction::Run);
        assert_eq!(cli.automation(), AutomationMode::None);
        assert_eq!(cli.frame_source(), FrameSourceKind::Real);
    }

    #[test]
    fn parses_synthetic_automatic_launch() {
        let cli = Cli::parse(arguments(&[
            "--frame-source",
            "synthetic",
            "--automation-exit",
        ]))
        .unwrap_or_else(|error| unreachable!("{error}"));

        assert_eq!(cli.action(), CliAction::Run);
        assert_eq!(cli.automation(), AutomationMode::ExitAfterLaunch);
        assert_eq!(cli.frame_source(), FrameSourceKind::Synthetic);
    }

    #[test]
    fn parses_screenshot_path_without_manual_input() {
        let cli = Cli::parse(arguments(&[
            "--frame-source",
            "synthetic",
            "--screenshot",
            "artifacts/capture.ppm",
        ]))
        .unwrap_or_else(|error| unreachable!("{error}"));

        assert_eq!(
            cli.screenshot(),
            Some(std::path::Path::new("artifacts/capture.ppm"))
        );
    }

    #[test]
    fn parses_paired_scripted_automation_paths() {
        let cli = Cli::parse(arguments(&[
            "--frame-source",
            "synthetic",
            "--automation-script",
            "fixtures/automation/full-smoke.json",
            "--automation-output",
            "artifacts/smoke",
        ]))
        .unwrap_or_else(|error| unreachable!("{error}"));

        assert_eq!(cli.automation(), AutomationMode::Scripted);
        assert_eq!(
            cli.automation_script(),
            Some(std::path::Path::new("fixtures/automation/full-smoke.json"))
        );
        assert_eq!(
            cli.automation_output(),
            Some(std::path::Path::new("artifacts/smoke"))
        );
    }

    #[test]
    fn rejects_unpaired_or_conflicting_scripted_automation() {
        assert_eq!(
            Cli::parse(arguments(&["--automation-script", "fixture.json",])),
            Err(CliError::ConflictingOptions)
        );
        assert_eq!(
            Cli::parse(arguments(&[
                "--automation-script",
                "fixture.json",
                "--automation-output",
                "output",
                "--screenshot",
                "capture.ppm",
            ])),
            Err(CliError::ConflictingOptions)
        );
    }

    #[test]
    fn parses_isolated_profile_directory() {
        let cli = Cli::parse(arguments(&["--profile-dir", "artifacts/test-profile"]))
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert_eq!(
            cli.profile_directory(),
            Some(std::path::Path::new("artifacts/test-profile"))
        );
    }

    #[test]
    fn rejects_missing_frame_source() {
        assert_eq!(
            Cli::parse(arguments(&["--frame-source"])),
            Err(CliError::MissingValue("--frame-source"))
        );
    }

    #[test]
    fn rejects_unknown_argument() {
        assert_eq!(
            Cli::parse(arguments(&["--camera"])),
            Err(CliError::UnknownArgument("--camera".to_owned()))
        );
    }

    #[test]
    fn rejects_conflicting_actions() {
        assert_eq!(
            Cli::parse(arguments(&["--help", "--version"])),
            Err(CliError::ConflictingOptions)
        );
    }
}
