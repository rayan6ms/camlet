//! Validated, deterministic packaged-style automation scenarios.

use std::collections::VecDeque;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use camlet_core::appearance::{OverlayShape, PreviewFitMode, ThemeId};
use camlet_core::settings::CameraResolution;
use serde::Deserialize;

const MAX_ACTIONS: usize = 10_000;
const MAX_REPEAT: u16 = 1_000;
const MAX_DELAY: Duration = Duration::from_secs(30 * 60);

/// One validated action consumed by the Iced update loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutomationAction {
    /// Confirm that a preview frame exists before continuing.
    WaitForPreview,
    /// Select a built-in theme.
    SetTheme(ThemeId),
    /// Select an overlay shape.
    SetShape(OverlayShape),
    /// Select camera fitting behavior.
    SetFit(PreviewFitMode),
    /// Select a ring width.
    SetRingThickness(u8),
    /// Select a corner radius.
    SetCornerRoundness(u8),
    /// Select a camera capture resolution.
    SetCameraResolution(CameraResolution),
    /// Resize by one product step.
    ResizeStep { grow: bool },
    /// Restart the selected camera.
    RestartCamera,
    /// Stop capture and resume it after the delay.
    SuspendResume(Duration),
    /// Wait while the application and capture continue running.
    Delay(Duration),
    /// Open the root context menu through the production window path.
    OpenMenu,
    /// Open the advanced submenu through the production window path.
    OpenAdvancedMenu,
    /// Capture the real compositor output into this validated relative filename.
    Screenshot(String),
    /// Capture the open context-menu window into this validated relative filename.
    MenuScreenshot(String),
    /// Export redacted diagnostics into this validated relative filename.
    Diagnostics(String),
    /// Write completion evidence and shut down.
    Quit,
}

/// Mutable scenario cursor and its isolated output directory.
#[derive(Debug, Clone)]
pub struct AutomationSession {
    actions: VecDeque<AutomationAction>,
    output_directory: PathBuf,
    started: bool,
}

impl AutomationSession {
    /// Loads, validates, and expands an automation fixture.
    ///
    /// # Errors
    ///
    /// Returns a path-redacted category for read, schema, or output failures.
    pub fn load(script_path: &Path, output_directory: &Path) -> Result<Self, AutomationError> {
        let bytes = fs::read(script_path).map_err(|_| AutomationError::Read)?;
        let script: Script = serde_json::from_slice(&bytes).map_err(|_| AutomationError::Schema)?;
        let actions = validate_and_expand(script)?;
        fs::create_dir_all(output_directory).map_err(|_| AutomationError::Output)?;
        Ok(Self {
            actions,
            output_directory: output_directory.to_path_buf(),
            started: false,
        })
    }

    /// Marks the first usable preview and writes the external readiness contract once.
    ///
    /// # Errors
    ///
    /// Returns an output error if the readiness marker cannot be written.
    pub fn start(&mut self) -> Result<bool, AutomationError> {
        if self.started {
            return Ok(false);
        }
        self.started = true;
        self.write(
            "ready.json",
            b"{\n  \"schemaVersion\": 1,\n  \"status\": \"ready\"\n}\n",
        )?;
        Ok(true)
    }

    /// Takes the next action.
    pub fn next(&mut self) -> Option<AutomationAction> {
        self.actions.pop_front()
    }

    /// Returns an incomplete action to the front of the scenario queue.
    pub fn retry(&mut self, action: AutomationAction) {
        self.actions.push_front(action);
    }

    /// Returns whether the readiness marker has been written.
    #[must_use]
    pub const fn started(&self) -> bool {
        self.started
    }

    /// Resolves a validated artifact filename inside the output directory.
    #[must_use]
    pub fn output_path(&self, filename: &str) -> PathBuf {
        self.output_directory.join(filename)
    }

    /// Writes a named artifact inside the output directory.
    ///
    /// # Errors
    ///
    /// Returns an output error if the file cannot be written.
    pub fn write(&self, filename: &str, contents: &[u8]) -> Result<(), AutomationError> {
        fs::write(self.output_path(filename), contents).map_err(|_| AutomationError::Output)
    }

    /// Writes successful completion evidence.
    ///
    /// # Errors
    ///
    /// Returns an output error if the marker cannot be written.
    pub fn complete(&self) -> Result<(), AutomationError> {
        self.write(
            "complete.json",
            b"{\n  \"schemaVersion\": 1,\n  \"status\": \"complete\"\n}\n",
        )
    }

    /// Writes stable failure evidence without private error details.
    pub fn fail(&self) {
        let _ = self.write(
            "failed.json",
            b"{\n  \"schemaVersion\": 1,\n  \"status\": \"failed\"\n}\n",
        );
    }
}

/// Privacy-safe automation configuration failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AutomationError {
    /// Fixture could not be read.
    #[error("automation fixture could not be read")]
    Read,
    /// Fixture did not match the bounded schema.
    #[error("automation fixture is invalid")]
    Schema,
    /// Artifact directory or file could not be written.
    #[error("automation output could not be written")]
    Output,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Script {
    schema_version: u32,
    actions: Vec<ScriptAction>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case", deny_unknown_fields)]
enum ScriptAction {
    WaitForPreview,
    SetTheme { value: ThemeId },
    SetShape { value: OverlayShape },
    SetFit { value: PreviewFitMode },
    SetRingThickness { value: u8 },
    SetCornerRoundness { value: u8 },
    SetCameraResolution { value: CameraResolution },
    ResizeCycle { repetitions: u16 },
    RestartCamera { repetitions: u16 },
    SuspendResume { milliseconds: u64 },
    Delay { milliseconds: u64 },
    OpenMenu,
    OpenAdvancedMenu,
    Screenshot { file: String },
    MenuScreenshot { file: String },
    Diagnostics { file: String },
    Quit,
}

fn validate_and_expand(script: Script) -> Result<VecDeque<AutomationAction>, AutomationError> {
    if script.schema_version != 1 || script.actions.is_empty() {
        return Err(AutomationError::Schema);
    }
    let mut actions = VecDeque::new();
    for action in script.actions {
        match action {
            ScriptAction::WaitForPreview => actions.push_back(AutomationAction::WaitForPreview),
            ScriptAction::SetTheme { value } => {
                actions.push_back(AutomationAction::SetTheme(value));
            }
            ScriptAction::SetShape { value } => {
                actions.push_back(AutomationAction::SetShape(value));
            }
            ScriptAction::SetFit { value } => {
                actions.push_back(AutomationAction::SetFit(value));
            }
            ScriptAction::SetRingThickness { value } if value <= 24 => {
                actions.push_back(AutomationAction::SetRingThickness(value));
            }
            ScriptAction::SetCornerRoundness { value } if value <= 72 => {
                actions.push_back(AutomationAction::SetCornerRoundness(value));
            }
            ScriptAction::SetCameraResolution { value } => {
                actions.push_back(AutomationAction::SetCameraResolution(value));
            }
            ScriptAction::ResizeCycle { repetitions } if repetitions <= MAX_REPEAT => {
                for _ in 0..repetitions {
                    actions.push_back(AutomationAction::ResizeStep { grow: true });
                    actions.push_back(AutomationAction::ResizeStep { grow: false });
                }
            }
            ScriptAction::RestartCamera { repetitions } if repetitions <= MAX_REPEAT => {
                actions.extend(std::iter::repeat_n(
                    AutomationAction::RestartCamera,
                    usize::from(repetitions),
                ));
            }
            ScriptAction::SuspendResume { milliseconds } => {
                actions.push_back(AutomationAction::SuspendResume(validate_delay(
                    milliseconds,
                )?));
            }
            ScriptAction::Delay { milliseconds } => {
                actions.push_back(AutomationAction::Delay(validate_delay(milliseconds)?));
            }
            ScriptAction::OpenMenu => actions.push_back(AutomationAction::OpenMenu),
            ScriptAction::OpenAdvancedMenu => {
                actions.push_back(AutomationAction::OpenAdvancedMenu);
            }
            ScriptAction::Screenshot { file } if valid_filename(&file) => {
                actions.push_back(AutomationAction::Screenshot(file));
            }
            ScriptAction::MenuScreenshot { file } if valid_filename(&file) => {
                actions.push_back(AutomationAction::MenuScreenshot(file));
            }
            ScriptAction::Diagnostics { file } if valid_filename(&file) => {
                actions.push_back(AutomationAction::Diagnostics(file));
            }
            ScriptAction::Quit => actions.push_back(AutomationAction::Quit),
            _ => return Err(AutomationError::Schema),
        }
        if actions.len() > MAX_ACTIONS {
            return Err(AutomationError::Schema);
        }
    }
    if !matches!(actions.front(), Some(AutomationAction::WaitForPreview))
        || !matches!(actions.back(), Some(AutomationAction::Quit))
        || actions
            .iter()
            .filter(|action| matches!(action, AutomationAction::Quit))
            .count()
            != 1
    {
        return Err(AutomationError::Schema);
    }
    Ok(actions)
}

fn validate_delay(milliseconds: u64) -> Result<Duration, AutomationError> {
    let duration = Duration::from_millis(milliseconds);
    if duration > MAX_DELAY {
        Err(AutomationError::Schema)
    } else {
        Ok(duration)
    }
}

fn valid_filename(filename: &str) -> bool {
    let path = Path::new(filename);
    !filename.is_empty()
        && filename.len() <= 128
        && path.components().count() == 1
        && matches!(path.components().next(), Some(Component::Normal(_)))
}

#[cfg(test)]
mod tests {
    use super::{AutomationAction, AutomationError, AutomationSession};

    #[test]
    fn valid_fixture_expands_cycles_and_writes_lifecycle_markers() {
        let directory = tempfile::tempdir().unwrap_or_else(|error| unreachable!("{error}"));
        let fixture = directory.path().join("fixture.json");
        std::fs::write(
            &fixture,
            r#"{
                "schemaVersion": 1,
                "actions": [
                    {"type":"wait-for-preview"},
                    {"type":"resize-cycle","repetitions":2},
                    {"type":"screenshot","file":"overlay.ppm"},
                    {"type":"quit"}
                ]
            }"#,
        )
        .unwrap_or_else(|error| unreachable!("{error}"));
        let output = directory.path().join("output");
        let mut session = AutomationSession::load(&fixture, &output)
            .unwrap_or_else(|error| unreachable!("{error}"));

        assert!(
            session
                .start()
                .unwrap_or_else(|error| unreachable!("{error}"))
        );
        assert!(
            !session
                .start()
                .unwrap_or_else(|error| unreachable!("{error}"))
        );
        assert!(output.join("ready.json").is_file());
        assert_eq!(session.next(), Some(AutomationAction::WaitForPreview));
        session.retry(AutomationAction::WaitForPreview);
        assert_eq!(session.next(), Some(AutomationAction::WaitForPreview));
        assert_eq!(
            session.next(),
            Some(AutomationAction::ResizeStep { grow: true })
        );
        assert_eq!(
            session.next(),
            Some(AutomationAction::ResizeStep { grow: false })
        );
        session
            .complete()
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(output.join("complete.json").is_file());
    }

    #[test]
    fn rejects_traversal_unbounded_delay_and_missing_quit() {
        let directory = tempfile::tempdir().unwrap_or_else(|error| unreachable!("{error}"));
        for (name, action) in [
            (
                "traversal",
                r#"{"type":"screenshot","file":"../private.ppm"}"#,
            ),
            ("delay", r#"{"type":"delay","milliseconds":1800001}"#),
        ] {
            let fixture = directory.path().join(format!("{name}.json"));
            std::fs::write(
                &fixture,
                format!(
                    "{{\"schemaVersion\":1,\"actions\":[{{\"type\":\"wait-for-preview\"}},{action},{{\"type\":\"quit\"}}]}}"
                ),
            )
            .unwrap_or_else(|error| unreachable!("{error}"));
            assert_eq!(
                AutomationSession::load(&fixture, &directory.path().join(name)).map(|_| ()),
                Err(AutomationError::Schema)
            );
        }
    }
}
