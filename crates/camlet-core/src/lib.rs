#![doc = "Pure domain types and behavior for Camlet."]

pub mod appearance;
pub mod geometry;
pub mod language;
pub mod menu;
pub mod settings;
pub mod state;

/// Human-readable product name.
pub const APP_NAME: &str = "Camlet";

/// Stable application identifier used for settings and packaging.
pub const APP_ID: &str = "dev.rayan.camlet";

/// Default logical width and height of the overlay.
pub const DEFAULT_OVERLAY_SIZE: u16 = 224;

/// Identifies the frame producer selected at startup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameSourceKind {
    /// Use the operating system's camera service.
    Real,
    /// Use Camlet's deterministic, privacy-safe test pattern.
    Synthetic,
}

impl FrameSourceKind {
    /// Parses the stable command-line spelling of a frame source.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "real" => Some(Self::Real),
            "synthetic" => Some(Self::Synthetic),
            _ => None,
        }
    }

    /// Returns the stable command-line spelling of this source.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Real => "real",
            Self::Synthetic => "synthetic",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FrameSourceKind;

    #[test]
    fn frame_source_names_round_trip() {
        for source in [FrameSourceKind::Real, FrameSourceKind::Synthetic] {
            assert_eq!(FrameSourceKind::parse(source.as_str()), Some(source));
        }
    }

    #[test]
    fn unknown_frame_source_is_rejected() {
        assert_eq!(FrameSourceKind::parse("camera"), None);
    }
}
