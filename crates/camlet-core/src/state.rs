//! Deterministic application state transitions and requested side effects.

use serde::{Deserialize, Serialize};

use crate::appearance::{
    AppearanceSettings, OverlayShape, PreviewFitMode, ThemeId, normalize_ring_thickness,
};
use crate::geometry::{RESIZE_STEP, WindowState, move_window, resize_square_window};
use crate::language::AppLanguage;
use crate::settings::{
    AppSettings, CAMERA_FPS_OPTIONS, CAMERA_RESOLUTION_OPTIONS, CameraResolution,
};

/// Camera lifecycle states shown by Camlet.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CameraStatus {
    /// Device enumeration or capture startup is in progress.
    #[default]
    Loading,
    /// Frames are being received.
    Preview,
    /// The user or system denied access.
    PermissionDenied,
    /// Another process or backend owns the device.
    CameraInUse,
    /// Enumeration returned no cameras.
    NoCamera,
    /// A saved camera could not be found.
    SelectedDeviceUnavailable,
    /// Unclassified backend failure.
    Error,
}

/// Camera entry safe for product UI. IDs must not enter diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraOption {
    /// Opaque backend identifier.
    pub id: String,
    /// User-visible name.
    pub label: String,
}

/// Entire testable product state, excluding frame pixels and adapter handles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    /// Persisted product state.
    pub settings: AppSettings,
    /// Current camera state.
    pub camera_status: CameraStatus,
    /// Most recently enumerated cameras.
    pub cameras: Vec<CameraOption>,
    /// Camera currently requested from the backend.
    pub active_camera_id: Option<String>,
    /// Whether resize controls are active.
    pub resize_mode: bool,
}

impl AppState {
    /// Creates application state from recovered settings.
    #[must_use]
    pub const fn new(settings: AppSettings) -> Self {
        Self {
            settings,
            camera_status: CameraStatus::Loading,
            cameras: Vec::new(),
            active_camera_id: None,
            resize_mode: false,
        }
    }

    /// Applies one product action and returns ordered adapter effects.
    #[must_use]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::SetTheme(theme) => {
                self.settings.appearance.apply_theme(theme);
                vec![Effect::PersistSettings]
            }
            Action::SetShape(shape) => {
                self.settings.appearance.shape = shape;
                vec![Effect::PersistSettings]
            }
            Action::SetFit(fit) => {
                self.settings.appearance.fit = fit;
                vec![Effect::PersistSettings]
            }
            Action::SetRingThickness(thickness) => {
                self.settings.appearance.ring_thickness =
                    normalize_ring_thickness(i64::from(thickness));
                vec![Effect::PersistSettings]
            }
            Action::SetCornerRoundness(roundness) => {
                self.settings.appearance.corner_roundness = roundness.min(72);
                vec![Effect::PersistSettings]
            }
            Action::SetLanguage(language) => {
                self.settings.language = language;
                vec![Effect::PersistSettings]
            }
            Action::SetCamera(id) => {
                if !self.cameras.iter().any(|camera| camera.id == id) {
                    return Vec::new();
                }
                self.settings.selected_camera_device_id = Some(id.clone());
                self.active_camera_id = Some(id.clone());
                self.camera_status = CameraStatus::Loading;
                vec![Effect::PersistSettings, Effect::StartCamera(id)]
            }
            Action::SetCameraFps(fps) => self.set_camera_fps(fps),
            Action::SetCameraResolution(resolution) => self.set_camera_resolution(resolution),
            Action::DevicesChanged(cameras) => self.replace_cameras(cameras),
            Action::CameraReady => {
                self.camera_status = CameraStatus::Preview;
                Vec::new()
            }
            Action::CameraFailed(status) => {
                self.camera_status = status;
                Vec::new()
            }
            Action::RetryCamera => {
                self.camera_status = CameraStatus::Loading;
                self.active_camera_id.clone().map_or_else(
                    || vec![Effect::EnumerateCameras],
                    |id| vec![Effect::StartCamera(id)],
                )
            }
            Action::ResetAppearance => {
                self.settings.appearance = AppearanceSettings::default();
                vec![Effect::PersistSettings]
            }
            Action::MoveWindow { x, y } => {
                self.settings.window = WindowState {
                    x,
                    y,
                    ..self.settings.window
                };
                vec![
                    Effect::MoveWindow(self.settings.window),
                    Effect::PersistSettings,
                ]
            }
            Action::NudgeWindow { x, y, accelerated } => {
                let multiplier = if accelerated {
                    i32::from(RESIZE_STEP)
                } else {
                    1
                };
                self.settings.window = move_window(
                    self.settings.window,
                    x.saturating_mul(multiplier),
                    y.saturating_mul(multiplier),
                );
                vec![
                    Effect::MoveWindow(self.settings.window),
                    Effect::PersistSettings,
                ]
            }
            Action::ResizeByStep { grow, maximum } => {
                let delta = if grow { RESIZE_STEP } else { -RESIZE_STEP };
                self.settings.window = resize_square_window(self.settings.window, delta, maximum);
                self.settings.appearance.size = self.settings.window.width.min(640);
                vec![
                    Effect::ResizeWindow(self.settings.window),
                    Effect::PersistSettings,
                ]
            }
            Action::SetResizeMode(enabled) => {
                self.resize_mode = enabled;
                Vec::new()
            }
            Action::OpenAbout => vec![Effect::OpenAbout],
            Action::Quit => vec![Effect::StopCamera, Effect::FlushSettings, Effect::Quit],
        }
    }

    fn set_camera_fps(&mut self, fps: u8) -> Vec<Effect> {
        if !CAMERA_FPS_OPTIONS.contains(&fps) || self.settings.camera_fps == fps {
            return Vec::new();
        }
        self.settings.camera_fps = fps;
        self.camera_status = CameraStatus::Loading;
        let restart = self
            .active_camera_id
            .clone()
            .map_or(Effect::EnumerateCameras, Effect::StartCamera);
        vec![Effect::PersistSettings, restart]
    }

    fn set_camera_resolution(&mut self, resolution: CameraResolution) -> Vec<Effect> {
        if !CAMERA_RESOLUTION_OPTIONS.contains(&resolution)
            || self.settings.camera_resolution == resolution
        {
            return Vec::new();
        }
        self.settings.camera_resolution = resolution;
        self.camera_status = CameraStatus::Loading;
        let restart = self
            .active_camera_id
            .clone()
            .map_or(Effect::EnumerateCameras, Effect::StartCamera);
        vec![Effect::PersistSettings, restart]
    }

    fn replace_cameras(&mut self, cameras: Vec<CameraOption>) -> Vec<Effect> {
        let previous_active = self.active_camera_id.clone();
        let previous_status = self.camera_status;
        self.cameras = cameras;
        if self.cameras.is_empty() {
            self.active_camera_id = None;
            self.camera_status = if self.settings.selected_camera_device_id.is_some() {
                CameraStatus::SelectedDeviceUnavailable
            } else {
                CameraStatus::NoCamera
            };
            return vec![Effect::StopCamera];
        }

        let saved = self.settings.selected_camera_device_id.as_deref();
        let selected = self
            .cameras
            .iter()
            .find(|camera| Some(camera.id.as_str()) == saved)
            .unwrap_or(&self.cameras[0])
            .id
            .clone();
        let changed = self.settings.selected_camera_device_id.as_deref() != Some(selected.as_str());
        self.settings.selected_camera_device_id = Some(selected.clone());
        self.active_camera_id = Some(selected.clone());

        if previous_active.as_deref() == Some(selected.as_str())
            && matches!(
                previous_status,
                CameraStatus::Loading | CameraStatus::Preview
            )
        {
            self.camera_status = previous_status;
            return if changed {
                vec![Effect::PersistSettings]
            } else {
                Vec::new()
            };
        }

        self.camera_status = CameraStatus::Loading;

        if changed {
            vec![Effect::PersistSettings, Effect::StartCamera(selected)]
        } else {
            vec![Effect::StartCamera(selected)]
        }
    }
}

/// User, window, and adapter events accepted by [`AppState`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Apply a built-in ring theme.
    SetTheme(ThemeId),
    /// Change the visible shape.
    SetShape(OverlayShape),
    /// Change camera frame fitting.
    SetFit(PreviewFitMode),
    /// Change ring width.
    SetRingThickness(u8),
    /// Change corner radius.
    SetCornerRoundness(u8),
    /// Change language selection.
    SetLanguage(AppLanguage),
    /// Select a camera by backend ID.
    SetCamera(String),
    /// Change the requested camera capture rate.
    SetCameraFps(u8),
    /// Change the requested camera capture resolution.
    SetCameraResolution(CameraResolution),
    /// Replace enumerated devices.
    DevicesChanged(Vec<CameraOption>),
    /// Capture produced its first usable frame.
    CameraReady,
    /// Capture entered an error state.
    CameraFailed(CameraStatus),
    /// Retry capture or enumeration.
    RetryCamera,
    /// Restore appearance defaults.
    ResetAppearance,
    /// Set absolute logical position.
    MoveWindow { x: i32, y: i32 },
    /// Move by one or 24 logical pixels.
    NudgeWindow { x: i32, y: i32, accelerated: bool },
    /// Resize the native overlay window by one menu step.
    ResizeByStep { grow: bool, maximum: u16 },
    /// Enter or leave resize mode.
    SetResizeMode(bool),
    /// Open About UI.
    OpenAbout,
    /// Cleanly stop and exit.
    Quit,
}

/// Side effects requested from the Iced/camera/storage adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Persist a debounced settings snapshot.
    PersistSettings,
    /// Flush settings before shutdown.
    FlushSettings,
    /// Enumerate camera devices.
    EnumerateCameras,
    /// Start or switch capture.
    StartCamera(String),
    /// Release capture.
    StopCamera,
    /// Move the native window.
    MoveWindow(WindowState),
    /// Resize and move the native window.
    ResizeWindow(WindowState),
    /// Open About UI.
    OpenAbout,
    /// Exit after prior effects complete.
    Quit,
}

#[cfg(test)]
mod tests {
    use super::{Action, AppState, CameraOption, CameraStatus, Effect};
    use crate::appearance::{
        CORNER_ROUNDNESS_OPTIONS, OverlayShape, PreviewFitMode, RING_THICKNESS_OPTIONS, ThemeId,
    };
    use crate::language::AppLanguage;
    use crate::settings::AppSettings;
    use crate::settings::{CAMERA_FPS_OPTIONS, CAMERA_RESOLUTION_OPTIONS, CameraResolution};

    #[test]
    fn first_device_is_selected_when_saved_device_disappears() {
        let settings = AppSettings {
            selected_camera_device_id: Some("missing".to_owned()),
            ..AppSettings::default()
        };
        let mut state = AppState::new(settings);
        let effects = state.update(Action::DevicesChanged(vec![CameraOption {
            id: "first".to_owned(),
            label: "Camera 1".to_owned(),
        }]));

        assert_eq!(state.active_camera_id.as_deref(), Some("first"));
        assert_eq!(
            effects,
            vec![
                Effect::PersistSettings,
                Effect::StartCamera("first".to_owned())
            ]
        );
    }

    #[test]
    fn empty_devices_preserve_saved_selection_for_future_reconnect() {
        let settings = AppSettings {
            selected_camera_device_id: Some("saved".to_owned()),
            ..AppSettings::default()
        };
        let mut state = AppState::new(settings);
        assert_eq!(
            state.update(Action::DevicesChanged(Vec::new())),
            vec![Effect::StopCamera]
        );
        assert_eq!(state.camera_status, CameraStatus::SelectedDeviceUnavailable);
        assert_eq!(
            state.settings.selected_camera_device_id.as_deref(),
            Some("saved")
        );
    }

    #[test]
    fn unchanged_periodic_enumeration_does_not_restart_preview() {
        let mut state = AppState::new(AppSettings::default());
        let cameras = vec![CameraOption {
            id: "stable".to_owned(),
            label: "Stable Camera".to_owned(),
        }];
        assert!(
            state
                .update(Action::DevicesChanged(cameras.clone()))
                .contains(&Effect::StartCamera("stable".to_owned()))
        );
        let _ = state.update(Action::CameraReady);

        assert!(state.update(Action::DevicesChanged(cameras)).is_empty());
        assert_eq!(state.camera_status, CameraStatus::Preview);
    }

    #[test]
    fn camera_failures_and_retry_preserve_the_selected_device() {
        let settings = AppSettings {
            selected_camera_device_id: Some("saved".to_owned()),
            ..AppSettings::default()
        };
        let mut state = AppState::new(settings);
        let _ = state.update(Action::DevicesChanged(vec![CameraOption {
            id: "saved".to_owned(),
            label: "Camera".to_owned(),
        }]));

        for status in [
            CameraStatus::PermissionDenied,
            CameraStatus::CameraInUse,
            CameraStatus::SelectedDeviceUnavailable,
            CameraStatus::Error,
        ] {
            assert!(state.update(Action::CameraFailed(status)).is_empty());
            assert_eq!(state.camera_status, status);
            assert_eq!(
                state.update(Action::RetryCamera),
                vec![Effect::StartCamera("saved".to_owned())]
            );
        }
    }

    #[test]
    fn changing_fps_persists_and_restarts_the_active_camera() {
        let mut state = AppState::new(AppSettings::default());
        state.active_camera_id = Some("camera".to_owned());

        assert_eq!(
            state.update(Action::SetCameraFps(60)),
            vec![
                Effect::PersistSettings,
                Effect::StartCamera("camera".to_owned())
            ]
        );
        assert_eq!(state.settings.camera_fps, 60);
        assert!(state.update(Action::SetCameraFps(59)).is_empty());
        assert_eq!(state.settings.camera_fps, 60);

        for fps in CAMERA_FPS_OPTIONS {
            let _ = state.update(Action::SetCameraFps(fps));
            assert_eq!(state.settings.camera_fps, fps);
        }
    }

    #[test]
    fn changing_resolution_persists_and_restarts_the_active_camera() {
        let mut state = AppState::new(AppSettings::default());
        state.active_camera_id = Some("camera".to_owned());

        assert_eq!(
            state.update(Action::SetCameraResolution(CameraResolution::R1280x720)),
            vec![
                Effect::PersistSettings,
                Effect::StartCamera("camera".to_owned())
            ]
        );
        assert_eq!(
            state.settings.camera_resolution,
            CameraResolution::R1280x720
        );

        for resolution in CAMERA_RESOLUTION_OPTIONS {
            let _ = state.update(Action::SetCameraResolution(resolution));
            assert_eq!(state.settings.camera_resolution, resolution);
        }
    }

    #[test]
    fn appearance_change_requests_persistence() {
        let mut state = AppState::new(AppSettings::default());
        assert_eq!(
            state.update(Action::SetShape(OverlayShape::Diamond)),
            vec![Effect::PersistSettings]
        );
        assert_eq!(state.settings.appearance.shape, OverlayShape::Diamond);

        assert_eq!(
            state.update(Action::SetTheme(ThemeId::Ocean)),
            vec![Effect::PersistSettings]
        );
        assert_eq!(state.settings.appearance.theme(), Some(ThemeId::Ocean));
    }

    #[test]
    fn quit_orders_cleanup_before_exit() {
        let mut state = AppState::new(AppSettings::default());
        assert_eq!(
            state.update(Action::Quit),
            vec![Effect::StopCamera, Effect::FlushSettings, Effect::Quit]
        );
    }

    #[test]
    fn every_static_appearance_and_language_choice_is_reachable() {
        let mut state = AppState::new(AppSettings::default());
        for theme in ThemeId::ALL {
            let _ = state.update(Action::SetTheme(theme));
            assert_eq!(state.settings.appearance.theme(), Some(theme));
        }
        for shape in OverlayShape::ALL {
            let _ = state.update(Action::SetShape(shape));
            assert_eq!(state.settings.appearance.shape, shape);
        }
        for fit in PreviewFitMode::ALL {
            let _ = state.update(Action::SetFit(fit));
            assert_eq!(state.settings.appearance.fit, fit);
        }
        for thickness in RING_THICKNESS_OPTIONS {
            let _ = state.update(Action::SetRingThickness(thickness));
            assert_eq!(state.settings.appearance.ring_thickness, thickness);
        }
        for roundness in CORNER_ROUNDNESS_OPTIONS {
            let _ = state.update(Action::SetCornerRoundness(roundness));
            assert_eq!(state.settings.appearance.corner_roundness, roundness);
        }
        for language in AppLanguage::ALL {
            let _ = state.update(Action::SetLanguage(language));
            assert_eq!(state.settings.language, language);
        }
    }
}
