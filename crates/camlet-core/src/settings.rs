//! Versioned native settings parsing and persistence.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use atomic_write_file::AtomicWriteFile;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::appearance::{
    AppearanceSettings, HexColor, MAXIMUM_OVERLAY_SIZE, MINIMUM_OVERLAY_SIZE, OverlayShape,
    PreviewFitMode, normalize_ring_thickness,
};
use crate::geometry::{MINIMUM_WINDOW_SIZE, WindowState};
use crate::language::AppLanguage;

/// Settings schema written by this native release.
pub const SETTINGS_SCHEMA_VERSION: u32 = 1;
/// Capture rates exposed by the context menu.
pub const CAMERA_FPS_OPTIONS: [u8; 4] = [15, 24, 30, 60];
/// Capture rate used when no valid saved value exists.
pub const DEFAULT_CAMERA_FPS: u8 = 30;
/// Capture resolutions exposed by the context menu.
pub const CAMERA_RESOLUTION_OPTIONS: [CameraResolution; 4] = [
    CameraResolution::R320x240,
    CameraResolution::R640x480,
    CameraResolution::R1280x720,
    CameraResolution::R1920x1080,
];

/// Persisted camera capture resolution.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CameraResolution {
    /// 320×240 low-bandwidth capture.
    #[serde(rename = "320x240")]
    R320x240,
    /// 640×480 standard-definition capture.
    #[default]
    #[serde(rename = "640x480")]
    R640x480,
    /// 1280×720 high-definition capture.
    #[serde(rename = "1280x720")]
    R1280x720,
    /// 1920×1080 full-HD capture.
    #[serde(rename = "1920x1080")]
    R1920x1080,
}

impl CameraResolution {
    /// Returns the requested physical capture dimensions.
    #[must_use]
    pub const fn dimensions(self) -> (u32, u32) {
        match self {
            Self::R320x240 => (320, 240),
            Self::R640x480 => (640, 480),
            Self::R1280x720 => (1_280, 720),
            Self::R1920x1080 => (1_920, 1_080),
        }
    }

    /// Returns the compact menu label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::R320x240 => "320 × 240",
            Self::R640x480 => "640 × 480",
            Self::R1280x720 => "1280 × 720",
            Self::R1920x1080 => "1920 × 1080",
        }
    }
}

/// Complete native settings document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// Schema version used to encode this document.
    pub schema_version: u32,
    /// Language selector.
    pub language: AppLanguage,
    /// Selected backend camera identifier.
    pub selected_camera_device_id: Option<String>,
    /// Requested camera capture rate in frames per second.
    pub camera_fps: u8,
    /// Requested camera capture resolution.
    pub camera_resolution: CameraResolution,
    /// Overlay appearance.
    pub appearance: AppearanceSettings,
    /// Last logical host-window bounds.
    pub window: WindowState,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            schema_version: SETTINGS_SCHEMA_VERSION,
            language: AppLanguage::System,
            selected_camera_device_id: None,
            camera_fps: DEFAULT_CAMERA_FPS,
            camera_resolution: CameraResolution::default(),
            appearance: AppearanceSettings::default(),
            window: WindowState::default(),
        }
    }
}

/// Result of tolerant field-by-field parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSettings {
    /// Recovered settings.
    pub settings: AppSettings,
    /// Stable names of fields that were present but invalid.
    pub repaired_fields: Vec<&'static str>,
    /// Schema version observed before recovery.
    pub source_schema_version: Option<u64>,
    /// Whether the source was syntactically valid JSON.
    pub valid_json: bool,
    /// Whether this native version may safely replace the source document.
    pub compatible_for_write: bool,
}

/// Result of loading the native file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsOrigin {
    /// Existing native file.
    Native,
    /// First launch without any settings file.
    Defaults,
}

/// Settings plus their load origin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedSettings {
    /// Parsed settings and repair metadata.
    pub parsed: ParsedSettings,
    /// Where those settings came from.
    pub origin: SettingsOrigin,
}

/// File-system or serialization failure while handling settings.
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    /// A file-system operation failed.
    #[error("settings I/O failed for {path}: {source}")]
    Io {
        /// Affected path.
        path: PathBuf,
        /// Underlying error.
        #[source]
        source: std::io::Error,
    },
    /// Native settings serialization failed.
    #[error("settings serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    /// The selected native path has no parent directory.
    #[error("settings path has no parent directory: {0}")]
    MissingParent(PathBuf),
}

/// Parses the current native JSON schema without rejecting unrelated valid fields.
#[must_use]
pub fn parse_settings_json(contents: &str) -> ParsedSettings {
    let Ok(value) = serde_json::from_str::<Value>(contents) else {
        return ParsedSettings {
            settings: AppSettings::default(),
            repaired_fields: vec!["document"],
            source_schema_version: None,
            valid_json: false,
            compatible_for_write: true,
        };
    };

    parse_settings_value(&value)
}

/// Loads native settings or returns defaults when no document exists.
///
/// # Errors
///
/// Returns an error when reading the settings document fails.
pub fn load_settings(native_path: &Path) -> Result<LoadedSettings, SettingsError> {
    if native_path.exists() {
        let bytes = read(native_path)?;
        return Ok(LoadedSettings {
            parsed: parse_settings_json(&String::from_utf8_lossy(&bytes)),
            origin: SettingsOrigin::Native,
        });
    }

    Ok(LoadedSettings {
        parsed: parse_settings_json("{}"),
        origin: SettingsOrigin::Defaults,
    })
}

/// Serializes and atomically replaces the native settings document.
///
/// # Errors
///
/// Returns an error when serialization or durable atomic replacement fails.
pub fn write_settings(path: &Path, settings: &AppSettings) -> Result<(), SettingsError> {
    let mut contents = serde_json::to_vec_pretty(settings)?;
    contents.push(b'\n');
    write_atomic_bytes(path, &contents)
}

fn parse_settings_value(value: &Value) -> ParsedSettings {
    let defaults = AppSettings::default();
    let Some(root) = value.as_object() else {
        return ParsedSettings {
            settings: defaults,
            repaired_fields: vec!["document"],
            source_schema_version: None,
            valid_json: true,
            compatible_for_write: true,
        };
    };
    let source_schema_version = root.get("schemaVersion").and_then(Value::as_u64);
    let compatible_for_write =
        source_schema_version.is_none_or(|version| version <= u64::from(SETTINGS_SCHEMA_VERSION));
    let mut repaired = Vec::new();

    let language = optional_string(root.get("language"))
        .and_then(parse_language)
        .unwrap_or_else(|| {
            mark_if_present(root.get("language"), "language", &mut repaired);
            defaults.language
        });
    let selected_camera_device_id =
        parse_device_id(root.get("selectedCameraDeviceId"), &mut repaired);
    let camera_fps_value = root.get("cameraFps");
    let camera_fps = camera_fps_value
        .and_then(Value::as_u64)
        .and_then(|value| u8::try_from(value).ok())
        .filter(|value| CAMERA_FPS_OPTIONS.contains(value))
        .unwrap_or_else(|| {
            mark_if_present(camera_fps_value, "cameraFps", &mut repaired);
            DEFAULT_CAMERA_FPS
        });
    let camera_resolution_value = root.get("cameraResolution");
    let camera_resolution = camera_resolution_value
        .and_then(Value::as_str)
        .and_then(parse_camera_resolution)
        .unwrap_or_else(|| {
            mark_if_present(camera_resolution_value, "cameraResolution", &mut repaired);
            CameraResolution::default()
        });
    let appearance = parse_appearance(root, &defaults.appearance, &mut repaired);
    let window = parse_window(root, &defaults.window, &mut repaired);

    ParsedSettings {
        settings: AppSettings {
            schema_version: SETTINGS_SCHEMA_VERSION,
            language,
            selected_camera_device_id,
            camera_fps,
            camera_resolution,
            appearance,
            window,
        },
        repaired_fields: repaired,
        source_schema_version,
        valid_json: true,
        compatible_for_write,
    }
}

fn parse_appearance(
    root: &Map<String, Value>,
    defaults: &AppearanceSettings,
    repaired: &mut Vec<&'static str>,
) -> AppearanceSettings {
    let appearance = root.get("appearance").and_then(Value::as_object);
    let shape_value = object_value(appearance, "shape");
    let shape = optional_string(shape_value)
        .and_then(parse_shape)
        .unwrap_or_else(|| {
            mark_if_present(shape_value, "appearance.shape", repaired);
            defaults.shape
        });
    let size_value = object_value(appearance, "size");
    let size =
        bounded_u16(size_value, MINIMUM_OVERLAY_SIZE, MAXIMUM_OVERLAY_SIZE).unwrap_or_else(|| {
            mark_if_present(size_value, "appearance.size", repaired);
            defaults.size
        });
    let ring_color_value = object_value(appearance, "ringColor");
    let ring_color = optional_string(ring_color_value)
        .and_then(HexColor::parse)
        .unwrap_or_else(|| {
            mark_if_present(ring_color_value, "appearance.ringColor", repaired);
            defaults.ring_color
        });
    let accent_value = object_value(appearance, "ringAccentColor");
    let ring_accent_color = optional_string(accent_value)
        .and_then(HexColor::parse)
        .unwrap_or_else(|| {
            mark_if_present(accent_value, "appearance.ringAccentColor", repaired);
            defaults.ring_accent_color
        });
    let ring_value = object_value(appearance, "ringThickness");
    let ring_thickness = ring_value.and_then(Value::as_i64).map_or_else(
        || {
            mark_if_present(ring_value, "appearance.ringThickness", repaired);
            defaults.ring_thickness
        },
        normalize_ring_thickness,
    );
    let roundness_value = object_value(appearance, "cornerRoundness");
    let corner_roundness = bounded_u8(roundness_value, 0, 72).unwrap_or_else(|| {
        mark_if_present(roundness_value, "appearance.cornerRoundness", repaired);
        defaults.corner_roundness
    });
    let fit_value = object_value(appearance, "fit");
    let fit = optional_string(fit_value)
        .and_then(parse_fit)
        .unwrap_or_else(|| {
            mark_if_present(fit_value, "appearance.fit", repaired);
            defaults.fit
        });

    AppearanceSettings {
        shape,
        size,
        ring_color,
        ring_accent_color,
        ring_thickness,
        corner_roundness,
        fit,
    }
}

fn parse_window(
    root: &Map<String, Value>,
    defaults: &WindowState,
    repaired: &mut Vec<&'static str>,
) -> WindowState {
    let window = root.get("window").and_then(Value::as_object);
    let x_value = object_value(window, "x");
    let y_value = object_value(window, "y");
    let width_value = object_value(window, "width");
    let height_value = object_value(window, "height");

    WindowState {
        x: signed_i32(x_value).unwrap_or_else(|| {
            mark_if_present(x_value, "window.x", repaired);
            defaults.x
        }),
        y: signed_i32(y_value).unwrap_or_else(|| {
            mark_if_present(y_value, "window.y", repaired);
            defaults.y
        }),
        width: bounded_u16(width_value, MINIMUM_WINDOW_SIZE, u16::MAX).unwrap_or_else(|| {
            mark_if_present(width_value, "window.width", repaired);
            defaults.width
        }),
        height: bounded_u16(height_value, MINIMUM_WINDOW_SIZE, u16::MAX).unwrap_or_else(|| {
            mark_if_present(height_value, "window.height", repaired);
            defaults.height
        }),
    }
}

fn parse_language(value: &str) -> Option<AppLanguage> {
    match value {
        "system" => Some(AppLanguage::System),
        "en" => Some(AppLanguage::English),
        "pt-BR" => Some(AppLanguage::PortugueseBrazil),
        _ => None,
    }
}

fn parse_shape(value: &str) -> Option<OverlayShape> {
    match value {
        "original" => Some(OverlayShape::Original),
        "circle" => Some(OverlayShape::Circle),
        "rounded-square" => Some(OverlayShape::RoundedSquare),
        "diamond" => Some(OverlayShape::Diamond),
        "rectangle-y" => Some(OverlayShape::RectangleY),
        "rectangle-x" => Some(OverlayShape::RectangleX),
        _ => None,
    }
}

fn parse_fit(value: &str) -> Option<PreviewFitMode> {
    match value {
        "cover" => Some(PreviewFitMode::Cover),
        "contain" => Some(PreviewFitMode::Contain),
        _ => None,
    }
}

fn parse_camera_resolution(value: &str) -> Option<CameraResolution> {
    match value {
        "320x240" => Some(CameraResolution::R320x240),
        "640x480" => Some(CameraResolution::R640x480),
        "1280x720" => Some(CameraResolution::R1280x720),
        "1920x1080" => Some(CameraResolution::R1920x1080),
        _ => None,
    }
}

fn parse_device_id(value: Option<&Value>, repaired: &mut Vec<&'static str>) -> Option<String> {
    match value {
        None | Some(Value::Null) => None,
        Some(Value::String(value)) if !value.trim().is_empty() => Some(value.clone()),
        Some(_) => {
            repaired.push("selectedCameraDeviceId");
            None
        }
    }
}

fn object_value<'a>(object: Option<&'a Map<String, Value>>, key: &str) -> Option<&'a Value> {
    object.and_then(|object| object.get(key))
}

fn optional_string(value: Option<&Value>) -> Option<&str> {
    value.and_then(Value::as_str)
}

fn bounded_u16(value: Option<&Value>, minimum: u16, maximum: u16) -> Option<u16> {
    let value = value?.as_u64()?;
    let value = u16::try_from(value).ok()?;
    (minimum..=maximum).contains(&value).then_some(value)
}

fn bounded_u8(value: Option<&Value>, minimum: u8, maximum: u8) -> Option<u8> {
    let value = value?.as_u64()?;
    let value = u8::try_from(value).ok()?;
    (minimum..=maximum).contains(&value).then_some(value)
}

fn signed_i32(value: Option<&Value>) -> Option<i32> {
    i32::try_from(value?.as_i64()?).ok()
}

fn mark_if_present(value: Option<&Value>, field: &'static str, repaired: &mut Vec<&'static str>) {
    if value.is_some() {
        repaired.push(field);
    }
}

fn create_dir_all(path: &Path) -> Result<(), SettingsError> {
    fs::create_dir_all(path).map_err(|source| SettingsError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn read(path: &Path) -> Result<Vec<u8>, SettingsError> {
    fs::read(path).map_err(|source| SettingsError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn write_atomic_bytes(path: &Path, contents: &[u8]) -> Result<(), SettingsError> {
    let parent = path
        .parent()
        .ok_or_else(|| SettingsError::MissingParent(path.to_path_buf()))?;
    create_dir_all(parent)?;
    let mut file = AtomicWriteFile::open(path).map_err(|source| SettingsError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    file.write_all(contents)
        .and_then(|()| file.commit())
        .map_err(|source| SettingsError::Io {
            path: path.to_path_buf(),
            source,
        })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use proptest::prelude::*;

    use super::{
        AppSettings, CAMERA_FPS_OPTIONS, CAMERA_RESOLUTION_OPTIONS, CameraResolution,
        DEFAULT_CAMERA_FPS, load_settings, parse_settings_json, write_settings,
    };
    use crate::appearance::{AppearanceSettings, HexColor, OverlayShape, PreviewFitMode};
    use crate::language::AppLanguage;

    #[test]
    fn invalid_fields_fall_back_without_dropping_valid_fields() {
        let parsed = parse_settings_json(
            r#"{
                "language": "pt-BR",
                "cameraFps": 144,
                "cameraResolution": "cinema",
                "appearance": {
                    "shape": "circle",
                    "size": 10,
                    "ringColor": "green",
                    "ringAccentColor": "pink",
                    "ringThickness": "huge",
                    "cornerRoundness": 400,
                    "fit": "stretch"
                }
            }"#,
        );

        assert_eq!(parsed.settings.language, AppLanguage::PortugueseBrazil);
        assert_eq!(parsed.settings.camera_fps, DEFAULT_CAMERA_FPS);
        assert_eq!(
            parsed.settings.camera_resolution,
            CameraResolution::default()
        );
        assert_eq!(
            parsed.settings.appearance,
            AppearanceSettings {
                shape: OverlayShape::Circle,
                ..AppearanceSettings::default()
            }
        );
        assert_eq!(parsed.repaired_fields.len(), 8);
    }

    #[test]
    fn native_document_round_trips() {
        let mut settings = AppSettings::default();
        settings.appearance.fit = PreviewFitMode::Contain;
        let json = serde_json::to_string(&settings).unwrap_or_else(|error| unreachable!("{error}"));
        let parsed = parse_settings_json(&json);
        assert_eq!(parsed.settings, settings);
        assert!(parsed.repaired_fields.is_empty());
    }

    #[test]
    fn versioned_fixture_matrix_recovers_deliberately() {
        let current =
            parse_settings_json(include_str!("../../../fixtures/settings/current-v1.json"));
        assert!(current.valid_json);
        assert!(current.compatible_for_write);
        assert_eq!(current.source_schema_version, Some(1));
        assert_eq!(current.settings.camera_fps, 60);
        assert_eq!(
            current.settings.camera_resolution,
            CameraResolution::R1280x720
        );
        assert_eq!(current.settings.appearance.shape, OverlayShape::Diamond);
        assert_eq!(current.settings.appearance.fit, PreviewFitMode::Contain);
        assert!(current.repaired_fields.is_empty());

        let partial = parse_settings_json(include_str!(
            "../../../fixtures/settings/partial-invalid.json"
        ));
        assert_eq!(partial.settings.language, AppLanguage::PortugueseBrazil);
        assert_eq!(partial.settings.camera_fps, DEFAULT_CAMERA_FPS);
        assert_eq!(
            partial.settings.appearance.ring_accent_color,
            HexColor::opaque(0x93_A1_B8)
        );
        assert!(partial.repaired_fields.len() >= 8);

        let future = parse_settings_json(include_str!(
            "../../../fixtures/settings/future-version.json"
        ));
        assert_eq!(future.source_schema_version, Some(999));
        assert!(!future.compatible_for_write);

        let corrupt = parse_settings_json(include_str!("../../../fixtures/settings/corrupt.txt"));
        assert!(!corrupt.valid_json);
        assert_eq!(corrupt.settings, AppSettings::default());
    }

    #[test]
    fn absent_file_loads_defaults_and_existing_file_loads_native_settings() {
        let directory = tempfile::tempdir().unwrap_or_else(|error| unreachable!("{error}"));
        let native = directory.path().join("settings.json");
        let first = load_settings(&native).unwrap_or_else(|error| unreachable!("{error}"));
        assert_eq!(first.parsed.settings, AppSettings::default());

        let settings = AppSettings {
            language: AppLanguage::PortugueseBrazil,
            ..AppSettings::default()
        };
        write_settings(&native, &settings).unwrap_or_else(|error| unreachable!("{error}"));
        let second = load_settings(&native).unwrap_or_else(|error| unreachable!("{error}"));
        assert_eq!(second.parsed.settings, settings);
    }

    #[test]
    fn atomic_writer_replaces_complete_document() {
        let directory = tempfile::tempdir().unwrap_or_else(|error| unreachable!("{error}"));
        let path = directory.path().join("settings.json");
        write_settings(&path, &AppSettings::default())
            .unwrap_or_else(|error| unreachable!("{error}"));
        let parsed = parse_settings_json(
            &fs::read_to_string(path).unwrap_or_else(|error| unreachable!("{error}")),
        );
        assert_eq!(parsed.settings, AppSettings::default());
    }

    proptest! {
        #[test]
        fn arbitrary_utf8_never_panics_or_escapes_bounds(input in ".*") {
            let parsed = parse_settings_json(&input);
            prop_assert!((96..=640).contains(&parsed.settings.appearance.size));
            prop_assert!(parsed.settings.appearance.ring_thickness <= 10);
            prop_assert!(parsed.settings.appearance.corner_roundness <= 72);
            prop_assert!(CAMERA_FPS_OPTIONS.contains(&parsed.settings.camera_fps));
            prop_assert!(CAMERA_RESOLUTION_OPTIONS.contains(&parsed.settings.camera_resolution));
            prop_assert!(parsed.settings.window.width >= 176);
            prop_assert!(parsed.settings.window.height >= 176);
        }
    }
}
