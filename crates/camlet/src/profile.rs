//! Application profile location and settings recovery.

use std::path::{Path, PathBuf};

use camlet_core::settings::{
    AppSettings, LoadedSettings, SettingsError, SettingsOrigin, load_settings,
};
use directories::BaseDirs;

/// Loaded settings plus the policy needed for future writes.
#[derive(Debug, Clone)]
pub struct NativeProfile {
    /// Recovered application settings.
    pub settings: AppSettings,
    /// Native settings document, if the platform exposes a config directory.
    pub settings_path: Option<PathBuf>,
    /// False for a future schema that this version must not overwrite.
    pub writable: bool,
    /// Stable description safe for diagnostics.
    pub origin: &'static str,
}

/// Resolves and loads the native profile.
///
/// An override keeps tests and automation isolated from the normal profile.
///
/// # Errors
///
/// Returns a typed settings error when the settings document cannot be read.
pub fn load(override_directory: Option<&Path>) -> Result<NativeProfile, SettingsError> {
    let settings_path = override_directory.map_or_else(
        || {
            BaseDirs::new().map_or_else(
                || None,
                |base| Some(base.config_dir().join("camlet").join("settings-v1.json")),
            )
        },
        |directory| Some(directory.join("settings-v1.json")),
    );

    let Some(settings_path) = settings_path else {
        return Ok(NativeProfile {
            settings: AppSettings::default(),
            settings_path: None,
            writable: false,
            origin: "defaults-no-config-directory",
        });
    };
    let loaded = load_settings(&settings_path)?;
    Ok(from_loaded(settings_path, loaded))
}

fn from_loaded(settings_path: PathBuf, loaded: LoadedSettings) -> NativeProfile {
    let origin = match loaded.origin {
        SettingsOrigin::Native => "native",
        SettingsOrigin::Defaults => "defaults",
    };
    NativeProfile {
        settings: loaded.parsed.settings,
        settings_path: Some(settings_path),
        writable: loaded.parsed.compatible_for_write,
        origin,
    }
}

#[cfg(test)]
mod tests {
    use camlet_core::appearance::OverlayShape;
    use camlet_core::settings::write_settings;

    use super::load;

    #[test]
    fn overridden_profile_is_isolated_and_persistent() {
        let directory = tempfile::tempdir().unwrap_or_else(|error| unreachable!("{error}"));
        let first = load(Some(directory.path())).unwrap_or_else(|error| unreachable!("{error}"));
        assert_eq!(first.origin, "defaults");
        let path = first.settings_path.unwrap_or_else(|| unreachable!());

        let mut settings = first.settings;
        settings.appearance.shape = OverlayShape::Diamond;
        write_settings(&path, &settings).unwrap_or_else(|error| unreachable!("{error}"));

        let second = load(Some(directory.path())).unwrap_or_else(|error| unreachable!("{error}"));
        assert_eq!(second.origin, "native");
        assert_eq!(second.settings.appearance.shape, OverlayShape::Diamond);
    }

    #[test]
    fn inaccessible_settings_document_returns_a_recoverable_error() {
        let directory = tempfile::tempdir().unwrap_or_else(|error| unreachable!("{error}"));
        std::fs::create_dir(directory.path().join("settings-v1.json"))
            .unwrap_or_else(|error| unreachable!("{error}"));

        assert!(load(Some(directory.path())).is_err());
    }
}
