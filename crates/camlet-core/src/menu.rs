//! UI-independent menu selection model.

use crate::appearance::{
    CORNER_ROUNDNESS_OPTIONS, OverlayShape, PreviewFitMode, RING_THICKNESS_OPTIONS, ThemeId,
};
use crate::language::AppLanguage;
use crate::settings::{CAMERA_FPS_OPTIONS, CAMERA_RESOLUTION_OPTIONS, CameraResolution};
use crate::state::{AppState, CameraOption, CameraStatus};

/// A selectable value and whether it matches current state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice<T> {
    /// Value dispatched when selected.
    pub value: T,
    /// Whether this is the active choice.
    pub selected: bool,
}

/// Camera choice with a user-visible label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraChoice {
    /// Opaque backend identifier.
    pub id: String,
    /// User-visible device label.
    pub label: String,
    /// Whether this device is active.
    pub selected: bool,
}

/// Complete values and enable/selected state needed to render the context menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuModel {
    /// Built-in themes.
    pub themes: Vec<Choice<ThemeId>>,
    /// Overlay shapes.
    pub shapes: Vec<Choice<OverlayShape>>,
    /// Language selectors.
    pub languages: Vec<Choice<AppLanguage>>,
    /// Enumerated camera choices.
    pub cameras: Vec<CameraChoice>,
    /// Preview fit choices.
    pub fit_modes: Vec<Choice<PreviewFitMode>>,
    /// Camera capture-rate choices.
    pub camera_fps: Vec<Choice<u8>>,
    /// Camera capture-resolution choices.
    pub camera_resolutions: Vec<Choice<CameraResolution>>,
    /// Ring widths.
    pub ring_thicknesses: Vec<Choice<u8>>,
    /// Corner radii.
    pub corner_roundnesses: Vec<Choice<u8>>,
    /// Current camera status.
    pub camera_status: CameraStatus,
    /// Whether resize mode is active.
    pub resize_mode: bool,
}

impl MenuModel {
    /// Derives a complete menu model from product state.
    #[must_use]
    pub fn from_state(state: &AppState) -> Self {
        Self {
            themes: choices(ThemeId::ALL, state.settings.appearance.theme()),
            shapes: choices(OverlayShape::ALL, Some(state.settings.appearance.shape)),
            languages: choices(AppLanguage::ALL, Some(state.settings.language)),
            cameras: state
                .cameras
                .iter()
                .map(|camera| camera_choice(camera, state.active_camera_id.as_deref()))
                .collect(),
            fit_modes: choices(PreviewFitMode::ALL, Some(state.settings.appearance.fit)),
            camera_fps: choices(CAMERA_FPS_OPTIONS, Some(state.settings.camera_fps)),
            camera_resolutions: choices(
                CAMERA_RESOLUTION_OPTIONS,
                Some(state.settings.camera_resolution),
            ),
            ring_thicknesses: choices(
                RING_THICKNESS_OPTIONS,
                Some(state.settings.appearance.ring_thickness),
            ),
            corner_roundnesses: choices(
                CORNER_ROUNDNESS_OPTIONS,
                Some(state.settings.appearance.corner_roundness),
            ),
            camera_status: state.camera_status,
            resize_mode: state.resize_mode,
        }
    }
}

fn choices<T: Copy + PartialEq>(
    values: impl IntoIterator<Item = T>,
    selected: Option<T>,
) -> Vec<Choice<T>> {
    values
        .into_iter()
        .map(|value| Choice {
            value,
            selected: Some(value) == selected,
        })
        .collect()
}

fn camera_choice(camera: &CameraOption, active: Option<&str>) -> CameraChoice {
    CameraChoice {
        id: camera.id.clone(),
        label: camera.label.clone(),
        selected: active == Some(camera.id.as_str()),
    }
}

#[cfg(test)]
mod tests {
    use super::MenuModel;
    use crate::settings::AppSettings;
    use crate::state::{AppState, CameraOption};

    #[test]
    fn default_model_has_exactly_one_selection_per_static_group() {
        let model = MenuModel::from_state(&AppState::new(AppSettings::default()));
        assert_eq!(
            model.themes.iter().filter(|choice| choice.selected).count(),
            1
        );
        assert_eq!(
            model.shapes.iter().filter(|choice| choice.selected).count(),
            1
        );
        assert_eq!(
            model
                .languages
                .iter()
                .filter(|choice| choice.selected)
                .count(),
            1
        );
        assert_eq!(
            model
                .fit_modes
                .iter()
                .filter(|choice| choice.selected)
                .count(),
            1
        );
        assert_eq!(
            model
                .ring_thicknesses
                .iter()
                .filter(|choice| choice.selected)
                .count(),
            1
        );
        assert_eq!(
            model
                .camera_fps
                .iter()
                .filter(|choice| choice.selected)
                .count(),
            1
        );
        assert_eq!(
            model
                .camera_resolutions
                .iter()
                .filter(|choice| choice.selected)
                .count(),
            1
        );
    }

    #[test]
    fn active_camera_selection_is_derived_without_logging() {
        let mut state = AppState::new(AppSettings::default());
        state.cameras = vec![CameraOption {
            id: "opaque-id".to_owned(),
            label: "Desk Camera".to_owned(),
        }];
        state.active_camera_id = Some("opaque-id".to_owned());
        let model = MenuModel::from_state(&state);
        assert!(model.cameras[0].selected);
        assert_eq!(model.cameras[0].label, "Desk Camera");
    }
}
