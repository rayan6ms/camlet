//! Overlay appearance values and validation.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::DEFAULT_OVERLAY_SIZE;

/// Minimum supported camera surface size in logical pixels.
pub const MINIMUM_OVERLAY_SIZE: u16 = 96;
/// Maximum supported camera surface size in logical pixels.
pub const MAXIMUM_OVERLAY_SIZE: u16 = 640;
/// Ring widths shown by the product menu.
pub const RING_THICKNESS_OPTIONS: [u8; 6] = [0, 2, 4, 6, 8, 10];
/// Corner radii shown by the product menu.
pub const CORNER_ROUNDNESS_OPTIONS: [u8; 6] = [0, 12, 24, 36, 48, 60];

/// Shape used to clip the preview and ring.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OverlayShape {
    /// The camera's original aspect-ratio rectangle without shape manipulation.
    #[default]
    Original,
    /// Elliptical surface; square bounds produce a circle.
    Circle,
    /// Square surface with configurable rounded corners.
    RoundedSquare,
    /// A square rotated by 45 degrees with configurable corners.
    Diamond,
    /// Portrait surface inset horizontally by 16%.
    RectangleY,
    /// Landscape surface inset vertically by 16%.
    RectangleX,
}

impl OverlayShape {
    /// All shapes in stable menu order.
    pub const ALL: [Self; 6] = [
        Self::Original,
        Self::Circle,
        Self::RoundedSquare,
        Self::RectangleY,
        Self::RectangleX,
        Self::Diamond,
    ];
}

/// How a source frame maps into the inner preview bounds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PreviewFitMode {
    /// Fill the viewport while preserving aspect ratio and cropping overflow.
    #[default]
    Cover,
    /// Show the complete frame while preserving aspect ratio.
    Contain,
}

impl PreviewFitMode {
    /// All fit modes in stable menu order.
    pub const ALL: [Self; 2] = [Self::Cover, Self::Contain];
}

/// Stable identifier for a built-in two-color ring theme.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeId {
    /// Mint green.
    #[default]
    Mint,
    /// Blue and cyan.
    Ocean,
    /// Orange and amber.
    Ember,
    /// Purple and pink.
    Orchid,
    /// Green and lime.
    Grove,
    /// White and blue gray.
    Graphite,
}

impl ThemeId {
    /// All themes in stable menu order.
    pub const ALL: [Self; 6] = [
        Self::Mint,
        Self::Ocean,
        Self::Ember,
        Self::Orchid,
        Self::Grove,
        Self::Graphite,
    ];

    /// Returns the immutable colors for this preset.
    #[must_use]
    pub const fn preset(self) -> ThemePreset {
        match self {
            Self::Mint => ThemePreset::new(Self::Mint, 0x7C_E2_C6, 0xC8_FF_F1),
            Self::Ocean => ThemePreset::new(Self::Ocean, 0x4D_A7_FF, 0x77_F1_E1),
            Self::Ember => ThemePreset::new(Self::Ember, 0xFF_6C_4D, 0xFF_B0_67),
            Self::Orchid => ThemePreset::new(Self::Orchid, 0xB7_8C_FF, 0xF1_A8_FF),
            Self::Grove => ThemePreset::new(Self::Grove, 0x57_D9_7B, 0xB8_FF_9D),
            Self::Graphite => ThemePreset::new(Self::Graphite, 0xF2_F5_F9, 0x93_A1_B8),
        }
    }
}

/// An RGBA color stored with exact 8-bit channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexColor {
    /// Red channel.
    pub red: u8,
    /// Green channel.
    pub green: u8,
    /// Blue channel.
    pub blue: u8,
    /// Alpha channel.
    pub alpha: u8,
}

impl HexColor {
    /// Creates an opaque color from a packed `0xRRGGBB` value.
    #[must_use]
    pub const fn opaque(value: u32) -> Self {
        let bytes = value.to_be_bytes();
        Self {
            red: bytes[1],
            green: bytes[2],
            blue: bytes[3],
            alpha: u8::MAX,
        }
    }

    /// Parses `#RRGGBB` or `#RRGGBBAA`.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        let digits = value.strip_prefix('#')?;
        if digits.len() != 6 && digits.len() != 8 {
            return None;
        }

        let red = u8::from_str_radix(&digits[0..2], 16).ok()?;
        let green = u8::from_str_radix(&digits[2..4], 16).ok()?;
        let blue = u8::from_str_radix(&digits[4..6], 16).ok()?;
        let alpha = if digits.len() == 8 {
            u8::from_str_radix(&digits[6..8], 16).ok()?
        } else {
            u8::MAX
        };

        Some(Self {
            red,
            green,
            blue,
            alpha,
        })
    }
}

impl fmt::Display for HexColor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.alpha == u8::MAX {
            write!(
                formatter,
                "#{:02X}{:02X}{:02X}",
                self.red, self.green, self.blue
            )
        } else {
            write!(
                formatter,
                "#{:02X}{:02X}{:02X}{:02X}",
                self.red, self.green, self.blue, self.alpha
            )
        }
    }
}

impl Serialize for HexColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for HexColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).ok_or_else(|| serde::de::Error::custom("expected #RRGGBB or #RRGGBBAA"))
    }
}

/// The immutable colors represented by a [`ThemeId`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemePreset {
    /// Stable theme identifier.
    pub id: ThemeId,
    /// Primary ring color.
    pub ring: HexColor,
    /// Secondary ring color.
    pub accent: HexColor,
}

impl ThemePreset {
    const fn new(id: ThemeId, ring: u32, accent: u32) -> Self {
        Self {
            id,
            ring: HexColor::opaque(ring),
            accent: HexColor::opaque(accent),
        }
    }
}

/// Persisted appearance state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppearanceSettings {
    /// Visible shape.
    pub shape: OverlayShape,
    /// Logical size of the square host surface.
    pub size: u16,
    /// Primary ring color.
    pub ring_color: HexColor,
    /// Secondary ring color.
    pub ring_accent_color: HexColor,
    /// Ring width in logical pixels.
    pub ring_thickness: u8,
    /// Shape corner radius in logical pixels.
    pub corner_roundness: u8,
    /// Camera frame fit behavior.
    pub fit: PreviewFitMode,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        let theme = ThemeId::Mint.preset();
        Self {
            shape: OverlayShape::Original,
            size: DEFAULT_OVERLAY_SIZE,
            ring_color: theme.ring,
            ring_accent_color: theme.accent,
            ring_thickness: 4,
            corner_roundness: 26,
            fit: PreviewFitMode::Cover,
        }
    }
}

impl AppearanceSettings {
    /// Applies a built-in theme without changing any other appearance value.
    pub const fn apply_theme(&mut self, id: ThemeId) {
        let preset = id.preset();
        self.ring_color = preset.ring;
        self.ring_accent_color = preset.accent;
    }

    /// Returns the matching built-in theme, or `None` for custom colors.
    #[must_use]
    pub fn theme(&self) -> Option<ThemeId> {
        ThemeId::ALL.into_iter().find(|id| {
            let preset = id.preset();
            preset.ring == self.ring_color && preset.accent == self.ring_accent_color
        })
    }
}

/// Clamps and snaps an arbitrary ring width to the nearest menu option.
#[must_use]
pub fn normalize_ring_thickness(value: i64) -> u8 {
    let clamped = value.clamp(0, 10);
    RING_THICKNESS_OPTIONS
        .into_iter()
        .min_by_key(|option| (i64::from(*option) - clamped).abs())
        .unwrap_or(4)
}

#[cfg(test)]
mod tests {
    use super::{AppearanceSettings, HexColor, OverlayShape, ThemeId, normalize_ring_thickness};

    #[test]
    fn original_camera_rectangle_is_the_default_shape() {
        assert_eq!(AppearanceSettings::default().shape, OverlayShape::Original);
    }

    #[test]
    fn colors_parse_and_serialize_canonically() {
        let color = HexColor::parse("#7ce2c6").unwrap_or_else(|| unreachable!());
        assert_eq!(color.to_string(), "#7CE2C6");
        assert_eq!(
            serde_json::to_string(&color).unwrap_or_else(|error| unreachable!("{error}")),
            "\"#7CE2C6\""
        );
        assert_eq!(HexColor::parse("green"), None);
    }

    #[test]
    fn every_theme_round_trips_through_appearance() {
        for id in ThemeId::ALL {
            let mut appearance = AppearanceSettings::default();
            appearance.apply_theme(id);
            assert_eq!(appearance.theme(), Some(id));
        }
    }

    #[test]
    fn arbitrary_ring_width_is_clamped_and_snapped() {
        assert_eq!(normalize_ring_thickness(-5), 0);
        assert_eq!(normalize_ring_thickness(3), 2);
        assert_eq!(normalize_ring_thickness(14), 10);
    }
}
