//! Pure overlay, window, and frame-fit geometry.

use serde::{Deserialize, Serialize};

use crate::DEFAULT_OVERLAY_SIZE;
use crate::appearance::{MAXIMUM_OVERLAY_SIZE, MINIMUM_OVERLAY_SIZE, OverlayShape, PreviewFitMode};

/// Minimum host window dimension in logical pixels.
pub const MINIMUM_WINDOW_SIZE: u16 = 176;
/// Keyboard/menu resize increment in logical pixels.
pub const RESIZE_STEP: i16 = 24;

/// Persisted logical window bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowState {
    /// Left screen coordinate.
    pub x: i32,
    /// Top screen coordinate.
    pub y: i32,
    /// Logical width.
    pub width: u16,
    /// Logical height.
    pub height: u16,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            x: 48,
            y: 48,
            width: DEFAULT_OVERLAY_SIZE,
            height: DEFAULT_OVERLAY_SIZE,
        }
    }
}

/// A monitor's usable logical bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkArea {
    /// Left screen coordinate.
    pub x: i32,
    /// Top screen coordinate.
    pub y: i32,
    /// Usable width.
    pub width: u32,
    /// Usable height.
    pub height: u32,
}

/// Floating-point rectangle used by rendering math.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// Horizontal origin.
    pub x: f64,
    /// Vertical origin.
    pub y: f64,
    /// Width.
    pub width: f64,
    /// Height.
    pub height: f64,
}

/// Source-to-destination mapping that preserves aspect ratio.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameTransform {
    /// Uniform scale applied to source coordinates.
    pub scale: f64,
    /// Horizontal destination offset after scaling.
    pub offset_x: f64,
    /// Vertical destination offset after scaling.
    pub offset_y: f64,
    /// Scaled frame width.
    pub width: f64,
    /// Scaled frame height.
    pub height: f64,
}

/// Clamps a stored window into a monitor work area.
#[must_use]
pub fn clamp_window_to_work_area(state: WindowState, area: WorkArea) -> WindowState {
    let area_width = u16::try_from(area.width).unwrap_or(u16::MAX);
    let area_height = u16::try_from(area.height).unwrap_or(u16::MAX);
    let minimum_width = MINIMUM_WINDOW_SIZE.min(area_width.max(1));
    let minimum_height = MINIMUM_WINDOW_SIZE.min(area_height.max(1));
    let width = state
        .width
        .clamp(minimum_width, area_width.max(minimum_width));
    let height = state
        .height
        .clamp(minimum_height, area_height.max(minimum_height));
    let max_x = i64::from(area.x) + i64::from(area.width) - i64::from(width);
    let max_y = i64::from(area.y) + i64::from(area.height) - i64::from(height);

    WindowState {
        x: clamp_i64_to_i32(i64::from(state.x).clamp(i64::from(area.x), max_x)),
        y: clamp_i64_to_i32(i64::from(state.y).clamp(i64::from(area.y), max_y)),
        width,
        height,
    }
}

/// Moves a window by a signed logical delta with saturating coordinates.
#[must_use]
pub const fn move_window(state: WindowState, delta_x: i32, delta_y: i32) -> WindowState {
    WindowState {
        x: state.x.saturating_add(delta_x),
        y: state.y.saturating_add(delta_y),
        ..state
    }
}

/// Resizes a square window while keeping its top-left corner stable.
///
/// Native window APIs generally expose move and resize as separate asynchronous operations.
/// Keeping the anchor stable lets the adapter resize in one operation without visible flicker.
#[must_use]
pub fn resize_square_window(state: WindowState, delta: i16, maximum: u16) -> WindowState {
    let maximum = maximum.max(MINIMUM_WINDOW_SIZE);
    let next = (i32::from(state.width) + i32::from(delta))
        .clamp(i32::from(MINIMUM_WINDOW_SIZE), i32::from(maximum));
    let size = u16::try_from(next).unwrap_or(MINIMUM_WINDOW_SIZE);
    WindowState {
        x: state.x,
        y: state.y,
        width: size,
        height: size,
    }
}

/// Returns the visible shape bounds inside its square host surface.
#[must_use]
pub fn shape_bounds(shape: OverlayShape, size: u16) -> Rect {
    shape_bounds_for_source(shape, size, 4, 3)
}

/// Returns visible shape bounds, preserving the source ratio for the original shape.
#[must_use]
pub fn shape_bounds_for_source(
    shape: OverlayShape,
    size: u16,
    source_width: u32,
    source_height: u32,
) -> Rect {
    let size = f64::from(size.clamp(MINIMUM_OVERLAY_SIZE, MAXIMUM_OVERLAY_SIZE));
    let inset = size * 0.16;

    match shape {
        OverlayShape::Original => {
            let aspect = if source_width == 0 || source_height == 0 {
                4.0 / 3.0
            } else {
                f64::from(source_width) / f64::from(source_height)
            };
            if aspect >= 1.0 {
                let height = size / aspect;
                Rect {
                    x: 0.0,
                    y: (size - height) / 2.0,
                    width: size,
                    height,
                }
            } else {
                let width = size * aspect;
                Rect {
                    x: (size - width) / 2.0,
                    y: 0.0,
                    width,
                    height: size,
                }
            }
        }
        OverlayShape::RectangleY => Rect {
            x: inset,
            y: 0.0,
            width: size - inset * 2.0,
            height: size,
        },
        OverlayShape::RectangleX => Rect {
            x: 0.0,
            y: inset,
            width: size,
            height: size - inset * 2.0,
        },
        OverlayShape::Circle | OverlayShape::RoundedSquare | OverlayShape::Diamond => Rect {
            x: 0.0,
            y: 0.0,
            width: size,
            height: size,
        },
    }
}

/// Computes a cover/contain transform for a frame and viewport.
#[must_use]
pub fn fit_frame(
    source_width: u32,
    source_height: u32,
    destination: Rect,
    mode: PreviewFitMode,
) -> Option<FrameTransform> {
    if source_width == 0
        || source_height == 0
        || destination.width <= 0.0
        || destination.height <= 0.0
    {
        return None;
    }

    let source_width = f64::from(source_width);
    let source_height = f64::from(source_height);
    let scale_x = destination.width / source_width;
    let scale_y = destination.height / source_height;
    let scale = match mode {
        PreviewFitMode::Cover => scale_x.max(scale_y),
        PreviewFitMode::Contain => scale_x.min(scale_y),
    };
    let width = source_width * scale;
    let height = source_height * scale;

    Some(FrameTransform {
        scale,
        offset_x: destination.x + (destination.width - width) / 2.0,
        offset_y: destination.y + (destination.height - height) / 2.0,
        width,
        height,
    })
}

fn clamp_i64_to_i32(value: i64) -> i32 {
    i32::try_from(value).unwrap_or_else(|_| {
        if value.is_negative() {
            i32::MIN
        } else {
            i32::MAX
        }
    })
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::{
        Rect, WindowState, WorkArea, clamp_window_to_work_area, fit_frame, resize_square_window,
        shape_bounds, shape_bounds_for_source,
    };
    use crate::appearance::{OverlayShape, PreviewFitMode};

    #[test]
    fn clamps_oversized_offscreen_window() {
        let result = clamp_window_to_work_area(
            WindowState {
                x: 4_000,
                y: -500,
                width: 900,
                height: 900,
            },
            WorkArea {
                x: 0,
                y: 0,
                width: 1_280,
                height: 720,
            },
        );
        assert_eq!(
            result,
            WindowState {
                x: 380,
                y: 0,
                width: 900,
                height: 720,
            }
        );
    }

    #[test]
    fn resize_keeps_stable_anchor_and_limit() {
        assert_eq!(
            resize_square_window(WindowState::default(), 800, 300),
            WindowState {
                x: 48,
                y: 48,
                width: 300,
                height: 300,
            }
        );
    }

    #[test]
    fn original_shape_preserves_landscape_and_portrait_source_ratios() {
        let landscape = shape_bounds_for_source(OverlayShape::Original, 240, 640, 480);
        assert_eq!(
            landscape,
            Rect {
                x: 0.0,
                y: 30.0,
                width: 240.0,
                height: 180.0
            }
        );

        let portrait = shape_bounds_for_source(OverlayShape::Original, 240, 480, 640);
        assert_eq!(
            portrait,
            Rect {
                x: 30.0,
                y: 0.0,
                width: 180.0,
                height: 240.0
            }
        );
    }

    #[test]
    fn portrait_shape_uses_reference_inset() {
        assert_eq!(
            shape_bounds(OverlayShape::RectangleY, 200),
            Rect {
                x: 32.0,
                y: 0.0,
                width: 136.0,
                height: 200.0,
            }
        );
    }

    #[test]
    fn fit_modes_preserve_aspect_ratio() {
        let destination = Rect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 200.0,
        };
        let cover = fit_frame(400, 200, destination, PreviewFitMode::Cover)
            .unwrap_or_else(|| unreachable!());
        let contain = fit_frame(400, 200, destination, PreviewFitMode::Contain)
            .unwrap_or_else(|| unreachable!());

        assert_eq!(
            (cover.width, cover.height, cover.offset_x),
            (400.0, 200.0, -100.0)
        );
        assert_eq!(
            (contain.width, contain.height, contain.offset_y),
            (200.0, 100.0, 50.0)
        );
    }

    #[test]
    fn every_shape_stays_inside_its_host_surface() {
        for shape in OverlayShape::ALL {
            for size in [96, 176, 224, 320, 480, 640] {
                let bounds = shape_bounds(shape, size);
                let size = f64::from(size);
                assert!(bounds.x >= 0.0 && bounds.y >= 0.0);
                assert!(bounds.x + bounds.width <= size);
                assert!(bounds.y + bounds.height <= size);
                assert!(bounds.width > 0.0 && bounds.height > 0.0);
            }
        }
    }

    proptest! {
        #[test]
        fn clamped_window_is_contained_in_any_positive_work_area(
            x in -5_000_i32..5_000,
            y in -5_000_i32..5_000,
            width in 1_u16..2_000,
            height in 1_u16..2_000,
            area_x in -1_000_i32..1_000,
            area_y in -1_000_i32..1_000,
            area_width in 1_u32..2_000,
            area_height in 1_u32..2_000,
        ) {
            let result = clamp_window_to_work_area(
                WindowState { x, y, width, height },
                WorkArea {
                    x: area_x,
                    y: area_y,
                    width: area_width,
                    height: area_height,
                },
            );
            let right = i64::from(result.x) + i64::from(result.width);
            let bottom = i64::from(result.y) + i64::from(result.height);
            prop_assert!(result.x >= area_x);
            prop_assert!(result.y >= area_y);
            prop_assert!(right <= i64::from(area_x) + i64::from(area_width));
            prop_assert!(bottom <= i64::from(area_y) + i64::from(area_height));
        }
    }
}
