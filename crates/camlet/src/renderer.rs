//! Deterministic premultiplied-alpha reference renderer.

use camlet_camera::VideoFrame;
use camlet_core::appearance::{AppearanceSettings, HexColor, OverlayShape};
use camlet_core::geometry::{Rect, fit_frame, shape_bounds_for_source};
use num_traits::ToPrimitive;

const EDGE_MARGIN_PHYSICAL_PIXELS: f64 = 2.0;

/// A complete premultiplied RGBA overlay produced by the test oracle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedFrame {
    /// Physical width.
    pub width: u32,
    /// Physical height.
    pub height: u32,
    /// Premultiplied sRGB RGBA bytes.
    pub rgba: Vec<u8>,
}

/// Invalid source or output geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RenderError {
    /// Output size was zero or exceeded the guarded implementation limit.
    #[error("invalid overlay output size")]
    InvalidOutputSize,
    /// Source dimensions and RGBA length disagreed.
    #[error("invalid source frame")]
    InvalidSourceFrame,
}

/// Renders one camera frame through the selected anti-aliased shape and ring.
///
/// # Errors
///
/// Returns a typed error for zero/oversized output or malformed source pixels.
pub fn render_overlay(
    source: &VideoFrame,
    appearance: &AppearanceSettings,
    physical_size: u32,
) -> Result<RenderedFrame, RenderError> {
    if physical_size == 0 || physical_size > 2_048 {
        return Err(RenderError::InvalidOutputSize);
    }
    validate_source_frame(source)?;
    let output_bytes =
        byte_len(physical_size, physical_size).ok_or(RenderError::InvalidOutputSize)?;
    let mut rgba = vec![0_u8; output_bytes];
    let surface_size = f64::from(physical_size);
    let logical_to_physical = surface_size / f64::from(appearance.size);
    let ring_width = f64::from(appearance.ring_thickness) * logical_to_physical;
    let corner_radius = f64::from(appearance.corner_roundness) * logical_to_physical;
    let visible_bounds = inset_rect(
        scale_rect(
            shape_bounds_for_source(
                appearance.shape,
                appearance.size,
                source.width,
                source.height,
            ),
            logical_to_physical,
        ),
        EDGE_MARGIN_PHYSICAL_PIXELS,
    );
    let frame_bounds = inset_rect(visible_bounds, ring_width.max(0.0));
    let frame_transform = fit_frame(source.width, source.height, frame_bounds, appearance.fit)
        .ok_or(RenderError::InvalidOutputSize)?;

    for y in 0..physical_size {
        for x in 0..physical_size {
            let point_x = f64::from(x) + 0.5;
            let point_y = f64::from(y) + 0.5;
            let distance = shape_distance(
                appearance.shape,
                point_x,
                point_y,
                visible_bounds,
                corner_radius,
            );
            let outer_coverage = coverage(distance);
            if outer_coverage <= 0.0 {
                continue;
            }
            let inner_coverage = coverage(distance + ring_width).min(outer_coverage);
            let ring_coverage = (outer_coverage - inner_coverage).max(0.0);
            let camera = sample_frame(source, frame_transform, point_x, point_y);
            let camera_alpha = f64::from(camera[3]) / 255.0 * inner_coverage;
            let gradient_position =
                ((point_x * 0.68 + point_y * 0.32) / surface_size).clamp(0.0, 1.0);
            let ring = interpolate_color(
                appearance.ring_color,
                appearance.ring_accent_color,
                gradient_position,
            );
            let alpha = (camera_alpha + ring_coverage).clamp(0.0, 1.0);
            let red = ring[0].mul_add(ring_coverage, f64::from(camera[0]) / 255.0 * camera_alpha);
            let green = ring[1].mul_add(ring_coverage, f64::from(camera[1]) / 255.0 * camera_alpha);
            let blue = ring[2].mul_add(ring_coverage, f64::from(camera[2]) / 255.0 * camera_alpha);
            let index = pixel_index(x, y, physical_size).ok_or(RenderError::InvalidOutputSize)?;
            rgba[index..index + 4].copy_from_slice(&[
                unit_to_byte(red),
                unit_to_byte(green),
                unit_to_byte(blue),
                unit_to_byte(alpha),
            ]);
        }
    }

    Ok(RenderedFrame {
        width: physical_size,
        height: physical_size,
        rgba,
    })
}

fn validate_source_frame(source: &VideoFrame) -> Result<(), RenderError> {
    let expected_source_bytes =
        byte_len(source.width, source.height).ok_or(RenderError::InvalidSourceFrame)?;
    if source.width == 0 || source.height == 0 || source.rgba.len() != expected_source_bytes {
        return Err(RenderError::InvalidSourceFrame);
    }
    Ok(())
}

fn byte_len(width: u32, height: u32) -> Option<usize> {
    usize::try_from(width)
        .ok()?
        .checked_mul(usize::try_from(height).ok()?)?
        .checked_mul(4)
}

fn pixel_index(x: u32, y: u32, width: u32) -> Option<usize> {
    let pixel = u64::from(y)
        .checked_mul(u64::from(width))?
        .checked_add(u64::from(x))?;
    usize::try_from(pixel.checked_mul(4)?).ok()
}

fn scale_rect(rectangle: Rect, scale: f64) -> Rect {
    Rect {
        x: rectangle.x * scale,
        y: rectangle.y * scale,
        width: rectangle.width * scale,
        height: rectangle.height * scale,
    }
}

fn inset_rect(rectangle: Rect, inset: f64) -> Rect {
    let maximum = (rectangle.width.min(rectangle.height) / 2.0 - 0.5).max(0.0);
    let inset = inset.clamp(0.0, maximum);
    Rect {
        x: rectangle.x + inset,
        y: rectangle.y + inset,
        width: (rectangle.width - inset * 2.0).max(1.0),
        height: (rectangle.height - inset * 2.0).max(1.0),
    }
}

fn shape_distance(shape: OverlayShape, x: f64, y: f64, host: Rect, corner_radius: f64) -> f64 {
    let center_x = host.x + host.width / 2.0;
    let center_y = host.y + host.height / 2.0;
    let local_x = x - center_x;
    let local_y = y - center_y;
    let half_width = host.width / 2.0;
    let half_height = host.height / 2.0;

    match shape {
        OverlayShape::Circle => local_x.hypot(local_y) - half_width.min(half_height),
        OverlayShape::Original
        | OverlayShape::RoundedSquare
        | OverlayShape::RectangleY
        | OverlayShape::RectangleX => {
            rounded_box_distance(local_x, local_y, half_width, half_height, corner_radius)
        }
        OverlayShape::Diamond => {
            let inverse_sqrt_two = std::f64::consts::FRAC_1_SQRT_2;
            let rotated_x = (local_x + local_y) * inverse_sqrt_two;
            let rotated_y = (-local_x + local_y) * inverse_sqrt_two;
            let half_side = half_width.min(half_height) * inverse_sqrt_two;
            rounded_box_distance(rotated_x, rotated_y, half_side, half_side, corner_radius)
        }
    }
}

fn rounded_box_distance(x: f64, y: f64, half_width: f64, half_height: f64, radius: f64) -> f64 {
    let radius = radius.clamp(0.0, half_width.min(half_height));
    let query_x = x.abs() - half_width + radius;
    let query_y = y.abs() - half_height + radius;
    let outside = query_x.max(0.0).hypot(query_y.max(0.0));
    let inside = query_x.max(query_y).min(0.0);
    outside + inside - radius
}

fn coverage(signed_distance: f64) -> f64 {
    let normalized = ((signed_distance + 1.25) / 2.5).clamp(0.0, 1.0);
    (normalized * normalized).mul_add(-2.0_f64.mul_add(-normalized, 3.0), 1.0)
}

fn sample_frame(
    source: &VideoFrame,
    transform: camlet_core::geometry::FrameTransform,
    x: f64,
    y: f64,
) -> [u8; 4] {
    if x < transform.offset_x
        || y < transform.offset_y
        || x >= transform.offset_x + transform.width
        || y >= transform.offset_y + transform.height
    {
        return [6, 10, 16, u8::MAX];
    }

    let source_x = ((x - transform.offset_x) / transform.scale)
        .floor()
        .clamp(0.0, f64::from(source.width.saturating_sub(1)));
    let source_y = ((y - transform.offset_y) / transform.scale)
        .floor()
        .clamp(0.0, f64::from(source.height.saturating_sub(1)));
    let mirrored_x = source
        .width
        .saturating_sub(1)
        .saturating_sub(source_x.to_u32().unwrap_or(0));
    let source_y = source_y.to_u32().unwrap_or(0);
    let Some(index) = pixel_index(mirrored_x, source_y, source.width) else {
        return [6, 10, 16, u8::MAX];
    };
    source
        .rgba
        .get(index..index + 4)
        .and_then(|pixel| <[u8; 4]>::try_from(pixel).ok())
        .unwrap_or([6, 10, 16, u8::MAX])
}

fn interpolate_color(start: HexColor, end: HexColor, position: f64) -> [f64; 3] {
    let mix = |first: u8, second: u8| {
        (f64::from(second) - f64::from(first)).mul_add(position, f64::from(first)) / 255.0
    };
    [
        mix(start.red, end.red),
        mix(start.green, end.green),
        mix(start.blue, end.blue),
    ]
}

fn unit_to_byte(value: f64) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0)
        .round()
        .to_u8()
        .unwrap_or(u8::MAX)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use camlet_camera::{CaptureRequest, FrameSource, SyntheticFrameSource, VideoFrame};
    use camlet_core::appearance::{
        AppearanceSettings, CORNER_ROUNDNESS_OPTIONS, OverlayShape, PreviewFitMode,
        RING_THICKNESS_OPTIONS,
    };
    use camlet_core::geometry::{Rect, fit_frame};

    use super::{RenderedFrame, render_overlay, sample_frame};

    fn source_frame() -> VideoFrame {
        let mut source = SyntheticFrameSource::default();
        source
            .start(
                None,
                CaptureRequest {
                    width: 96,
                    height: 72,
                    frame_interval: Duration::from_millis(33),
                },
            )
            .unwrap_or_else(|error| unreachable!("{error}"));
        source
            .latest_frame()
            .unwrap_or_else(|error| unreachable!("{error}"))
            .unwrap_or_else(|| unreachable!())
    }

    fn alpha(frame: &RenderedFrame, x: u32, y: u32) -> u8 {
        let index = usize::try_from((u64::from(y) * u64::from(frame.width) + u64::from(x)) * 4)
            .unwrap_or_else(|error| unreachable!("{error}"));
        frame.rgba[index + 3]
    }

    fn composite(frame: &RenderedFrame, x: u32, y: u32, background: [u8; 3]) -> [u8; 3] {
        let index = usize::try_from((u64::from(y) * u64::from(frame.width) + u64::from(x)) * 4)
            .unwrap_or_else(|error| unreachable!("{error}"));
        let alpha = u16::from(frame.rgba[index + 3]);
        std::array::from_fn(|channel| {
            let source = u16::from(frame.rgba[index + channel]);
            let behind = u16::from(background[channel]);
            u8::try_from(source + (behind * (255 - alpha) + 127) / 255)
                .unwrap_or_else(|error| unreachable!("{error}"))
        })
    }

    #[test]
    fn rejects_malformed_source() {
        let source = VideoFrame {
            width: 10,
            height: 10,
            sequence: 0,
            rgba: vec![0; 10],
        };
        assert!(render_overlay(&source, &AppearanceSettings::default(), 224).is_err());
    }

    #[test]
    fn circle_has_transparent_corners_and_opaque_center() {
        let appearance = AppearanceSettings {
            shape: OverlayShape::Circle,
            ..AppearanceSettings::default()
        };
        let frame = render_overlay(&source_frame(), &appearance, 224)
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert_eq!(alpha(&frame, 0, 0), 0);
        assert_eq!(alpha(&frame, 223, 0), 0);
        assert_eq!(alpha(&frame, 112, 112), u8::MAX);
        for pixel in frame.rgba.chunks_exact(4) {
            assert!(pixel[0] <= pixel[3]);
            assert!(pixel[1] <= pixel[3]);
            assert!(pixel[2] <= pixel[3]);
        }
    }

    #[test]
    fn original_shape_preserves_camera_ratio_with_rounded_corners() {
        let frame = render_overlay(&source_frame(), &AppearanceSettings::default(), 224)
            .unwrap_or_else(|error| unreachable!("{error}"));

        assert_eq!(alpha(&frame, 3, 31), 0);
        assert_eq!(alpha(&frame, 5, 56), u8::MAX);
        assert_eq!(alpha(&frame, 112, 33), u8::MAX);
        assert_eq!(alpha(&frame, 112, 190), u8::MAX);
    }

    #[test]
    fn circle_alpha_is_exactly_symmetric() {
        let appearance = AppearanceSettings {
            shape: OverlayShape::Circle,
            ..AppearanceSettings::default()
        };
        for size in [176, 224, 320, 640] {
            let frame = render_overlay(&source_frame(), &appearance, size)
                .unwrap_or_else(|error| unreachable!("{error}"));
            for coordinate in 0..size {
                assert_eq!(
                    alpha(&frame, coordinate, size / 2),
                    alpha(&frame, size - 1 - coordinate, size / 2)
                );
                assert_eq!(
                    alpha(&frame, size / 2, coordinate),
                    alpha(&frame, size / 2, size - 1 - coordinate)
                );
            }
        }
    }

    #[test]
    fn cardinal_edges_have_monotonic_coverage() {
        let appearance = AppearanceSettings {
            shape: OverlayShape::Circle,
            ..AppearanceSettings::default()
        };
        let frame = render_overlay(&source_frame(), &appearance, 640)
            .unwrap_or_else(|error| unreachable!("{error}"));
        let center = frame.width / 2;
        let mut previous = 0;
        for coordinate in 0..=center {
            let horizontal = alpha(&frame, coordinate, center);
            let vertical = alpha(&frame, center, coordinate);
            assert!(horizontal >= previous);
            assert_eq!(horizontal, vertical);
            previous = horizontal;
        }
    }

    #[test]
    fn circle_edge_fades_fully_inside_the_window() {
        let appearance = AppearanceSettings {
            shape: OverlayShape::Circle,
            ring_thickness: 0,
            ..AppearanceSettings::default()
        };
        let frame = render_overlay(&source_frame(), &appearance, 640)
            .unwrap_or_else(|error| unreachable!("{error}"));
        let center = frame.width / 2;
        assert_eq!(alpha(&frame, 0, center), 0);
        let transitional = (0..8)
            .map(|x| alpha(&frame, x, center))
            .filter(|value| (1..=254).contains(value))
            .count();
        assert!(transitional >= 2);
        assert_eq!(alpha(&frame, 8, center), u8::MAX);
    }

    #[test]
    fn all_required_sizes_scales_and_shapes_render_valid_alpha() {
        let source = source_frame();
        for logical_size in [176_u32, 224, 320, 480, 640] {
            for scale_numerator in [4_u32, 5, 6, 8] {
                let physical_size = logical_size * scale_numerator / 4;
                for shape in OverlayShape::ALL {
                    let appearance = AppearanceSettings {
                        shape,
                        size: u16::try_from(logical_size)
                            .unwrap_or_else(|error| unreachable!("{error}")),
                        ..AppearanceSettings::default()
                    };
                    let frame = render_overlay(&source, &appearance, physical_size)
                        .unwrap_or_else(|error| unreachable!("{error}"));
                    assert_eq!(
                        frame.rgba.len(),
                        usize::try_from(physical_size * physical_size * 4)
                            .unwrap_or_else(|error| unreachable!("{error}"))
                    );
                    assert_eq!(alpha(&frame, 0, 0), 0);
                    assert_eq!(alpha(&frame, physical_size / 2, physical_size / 2), u8::MAX);
                    for background in [
                        [255, 255, 255],
                        [0, 0, 0],
                        [127, 127, 127],
                        [40, 40, 40],
                        [255, 0, 0],
                        [0, 255, 0],
                        [0, 0, 255],
                    ] {
                        assert_eq!(composite(&frame, 0, 0, background), background);
                        assert_eq!(
                            composite(&frame, physical_size / 2, physical_size / 2, background,),
                            composite(&frame, physical_size / 2, physical_size / 2, [0, 0, 0],)
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn reviewed_shape_golden_checksums_are_stable() {
        let source = source_frame();
        let mut checksums = Vec::new();
        for shape in OverlayShape::ALL {
            let appearance = AppearanceSettings {
                shape,
                ..AppearanceSettings::default()
            };
            let frame = render_overlay(&source, &appearance, 224)
                .unwrap_or_else(|error| unreachable!("{error}"));
            let checksum = frame
                .rgba
                .iter()
                .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
                    (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
                });
            checksums.push(checksum);
        }

        assert_eq!(
            checksums,
            [
                0x44b0_7c8f_1537_763c,
                0xadc7_9bff_7bcf_6ea0,
                0x9e8c_249e_8633_b4fe,
                0x0fa4_5ce3_6396_68f7,
                0xea3c_89ec_d40b_802e,
                0xd3e2_3a53_a4d4_3cf7,
            ]
        );
    }

    #[test]
    fn all_appearance_geometry_options_render() {
        let source = source_frame();
        for shape in OverlayShape::ALL {
            for fit in PreviewFitMode::ALL {
                for thickness in RING_THICKNESS_OPTIONS {
                    for roundness in CORNER_ROUNDNESS_OPTIONS {
                        let appearance = AppearanceSettings {
                            shape,
                            fit,
                            ring_thickness: thickness,
                            corner_roundness: roundness,
                            ..AppearanceSettings::default()
                        };
                        let frame = render_overlay(&source, &appearance, 224)
                            .unwrap_or_else(|error| unreachable!("{error}"));
                        assert_eq!(frame.rgba.len(), 224 * 224 * 4);
                    }
                }
            }
        }
    }

    #[test]
    fn camera_sampling_is_mirrored_before_upload() {
        let source = VideoFrame {
            width: 2,
            height: 1,
            sequence: 0,
            rgba: vec![255, 0, 0, 255, 0, 0, 255, 255],
        };
        let transform = fit_frame(
            2,
            1,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 2.0,
                height: 1.0,
            },
            PreviewFitMode::Contain,
        )
        .unwrap_or_else(|| unreachable!());
        assert_eq!(sample_frame(&source, transform, 0.5, 0.5), [0, 0, 255, 255]);
        assert_eq!(sample_frame(&source, transform, 1.5, 0.5), [255, 0, 0, 255]);
    }
}
