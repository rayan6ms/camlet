//! Automation screenshot serialization with no image-decoder dependency.

use std::fs;
use std::path::{Path, PathBuf};

use iced::window::Screenshot;

/// Screenshot persistence failure.
#[derive(Debug, thiserror::Error)]
pub enum ScreenshotError {
    /// Screenshot dimensions and RGBA length disagreed.
    #[error("Iced returned malformed screenshot bytes")]
    InvalidBytes,
    /// The compositor capture did not preserve the overlay's transparent host corners.
    #[error("overlay screenshot has an opaque corner or transparent center")]
    InvalidTransparency,
    /// The live WGPU shader disagreed with the deterministic CPU alpha oracle.
    #[error("overlay screenshot differs from the reference renderer")]
    ReferenceMismatch,
    /// Target path has no parent directory.
    #[error("screenshot path has no parent: {0}")]
    MissingParent(PathBuf),
    /// File-system operation failed.
    #[error("could not write screenshot {path}: {source}")]
    Io {
        /// Affected path.
        path: PathBuf,
        /// Underlying error.
        #[source]
        source: std::io::Error,
    },
}

/// Compares every captured alpha value to a tightly packed premultiplied RGBA oracle.
///
/// # Errors
///
/// Returns [`ScreenshotError::ReferenceMismatch`] when dimensions differ or any alpha channel
/// exceeds the declared tolerance.
pub fn validate_reference_alpha(
    screenshot: &Screenshot,
    reference_width: u32,
    reference_height: u32,
    reference_rgba: &[u8],
    tolerance: u8,
) -> Result<(), ScreenshotError> {
    if screenshot.size.width != reference_width || screenshot.size.height != reference_height {
        return Err(ScreenshotError::ReferenceMismatch);
    }
    let pixel_count = usize::try_from(reference_width)
        .ok()
        .and_then(|width| {
            usize::try_from(reference_height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .ok_or(ScreenshotError::ReferenceMismatch)?;
    if screenshot.rgba.len() != pixel_count.saturating_mul(4)
        || reference_rgba.len() != pixel_count.saturating_mul(4)
        || screenshot
            .rgba
            .chunks_exact(4)
            .zip(reference_rgba.chunks_exact(4))
            .any(|(actual, expected)| actual[3].abs_diff(expected[3]) > tolerance)
    {
        return Err(ScreenshotError::ReferenceMismatch);
    }
    Ok(())
}

/// Verifies the minimum compositor contract required by the transparent overlay.
///
/// # Errors
///
/// Returns an error when the capture is malformed, any host corner is not transparent, or the
/// center of the visible preview is not opaque.
pub fn validate_transparent_overlay(screenshot: &Screenshot) -> Result<(), ScreenshotError> {
    let width =
        usize::try_from(screenshot.size.width).map_err(|_| ScreenshotError::InvalidBytes)?;
    let height =
        usize::try_from(screenshot.size.height).map_err(|_| ScreenshotError::InvalidBytes)?;
    let pixel_count = width
        .checked_mul(height)
        .ok_or(ScreenshotError::InvalidBytes)?;
    if width == 0 || height == 0 || screenshot.rgba.len() != pixel_count.saturating_mul(4) {
        return Err(ScreenshotError::InvalidBytes);
    }

    let alpha_at = |x: usize, y: usize| screenshot.rgba[(y * width + x) * 4 + 3];
    let corners = [
        alpha_at(0, 0),
        alpha_at(width - 1, 0),
        alpha_at(0, height - 1),
        alpha_at(width - 1, height - 1),
    ];
    if corners.into_iter().any(|alpha| alpha != 0) || alpha_at(width / 2, height / 2) < 250 {
        return Err(ScreenshotError::InvalidTransparency);
    }

    let center_x = width / 2;
    let center_y = height / 2;
    let cardinal_axes = [
        (0..=center_y)
            .map(|y| alpha_at(center_x, y))
            .collect::<Vec<_>>(),
        (center_y..height)
            .rev()
            .map(|y| alpha_at(center_x, y))
            .collect(),
        (0..=center_x).map(|x| alpha_at(x, center_y)).collect(),
        (center_x..width)
            .rev()
            .map(|x| alpha_at(x, center_y))
            .collect(),
    ];
    if cardinal_axes.iter().any(|axis| {
        axis.windows(2)
            .any(|pair| pair[1].saturating_add(1) < pair[0])
    }) {
        return Err(ScreenshotError::InvalidTransparency);
    }

    Ok(())
}

/// Composites an Iced RGBA capture over white and writes a portable binary PPM image.
///
/// # Errors
///
/// Returns a typed error for malformed pixels, invalid paths, or file-system failures.
pub fn write_white_background_ppm(
    path: &Path,
    screenshot: &Screenshot,
) -> Result<(), ScreenshotError> {
    let pixel_count = usize::try_from(screenshot.size.width)
        .ok()
        .and_then(|width| {
            usize::try_from(screenshot.size.height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .ok_or(ScreenshotError::InvalidBytes)?;
    if screenshot.rgba.len() != pixel_count.saturating_mul(4) {
        return Err(ScreenshotError::InvalidBytes);
    }
    let parent = path
        .parent()
        .ok_or_else(|| ScreenshotError::MissingParent(path.to_path_buf()))?;
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    fs::create_dir_all(parent).map_err(|source| ScreenshotError::Io {
        path: parent.to_path_buf(),
        source,
    })?;
    let header = format!(
        "P6\n# Camlet Iced screenshot; scale={}\n{} {}\n255\n",
        screenshot.scale_factor, screenshot.size.width, screenshot.size.height
    );
    let mut ppm = Vec::with_capacity(header.len().saturating_add(pixel_count.saturating_mul(3)));
    ppm.extend_from_slice(header.as_bytes());
    for pixel in screenshot.rgba.chunks_exact(4) {
        let alpha = u16::from(pixel[3]);
        for channel in &pixel[..3] {
            let composited = (u16::from(*channel) * alpha + 255 * (255 - alpha) + 127) / 255;
            ppm.push(u8::try_from(composited.min(255)).unwrap_or(u8::MAX));
        }
    }
    fs::write(path, ppm).map_err(|source| ScreenshotError::Io {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use iced::{Size, window::Screenshot};

    use super::{
        ScreenshotError, validate_reference_alpha, validate_transparent_overlay,
        write_white_background_ppm,
    };

    #[test]
    fn validates_transparent_corners_and_opaque_center() {
        let mut rgba = vec![0; 3 * 3 * 4];
        rgba[19] = u8::MAX;
        let screenshot = Screenshot::new(rgba, Size::new(3, 3), 1.0);
        assert!(validate_transparent_overlay(&screenshot).is_ok());
    }

    #[test]
    fn rejects_opaque_host_corner() {
        let mut rgba = vec![0; 3 * 3 * 4];
        rgba[3] = u8::MAX;
        rgba[19] = u8::MAX;
        let screenshot = Screenshot::new(rgba, Size::new(3, 3), 1.0);
        assert!(matches!(
            validate_transparent_overlay(&screenshot),
            Err(ScreenshotError::InvalidTransparency)
        ));
    }

    #[test]
    fn rejects_a_cardinal_axis_notch() {
        let size = 7_usize;
        let mut rgba = vec![0; size * size * 4];
        for y in 0..size {
            for x in 0..size {
                let distance = x.abs_diff(3).pow(2) + y.abs_diff(3).pow(2);
                if distance <= 9 {
                    rgba[(y * size + x) * 4 + 3] = u8::MAX;
                }
            }
        }
        rgba[(2 * size + 3) * 4 + 3] = 100;
        let screenshot = Screenshot::new(rgba, Size::new(7, 7), 1.0);
        assert!(matches!(
            validate_transparent_overlay(&screenshot),
            Err(ScreenshotError::InvalidTransparency)
        ));
    }

    #[test]
    fn writes_rgba_composited_over_white() {
        let directory = tempfile::tempdir().unwrap_or_else(|error| unreachable!("{error}"));
        let path = directory.path().join("capture.ppm");
        let screenshot = Screenshot::new(vec![255, 0, 0, 255, 0, 0, 0, 0], Size::new(2, 1), 1.0);
        write_white_background_ppm(&path, &screenshot)
            .unwrap_or_else(|error| unreachable!("{error}"));
        let bytes = std::fs::read(path).unwrap_or_else(|error| unreachable!("{error}"));
        assert!(bytes.ends_with(&[255, 0, 0, 255, 255, 255]));
    }

    #[test]
    fn reference_alpha_comparison_has_an_explicit_edge_tolerance() {
        let screenshot = Screenshot::new(vec![1, 2, 3, 128], Size::new(1, 1), 1.0);
        assert!(validate_reference_alpha(&screenshot, 1, 1, &[9, 8, 7, 126], 2).is_ok());
        assert!(matches!(
            validate_reference_alpha(&screenshot, 1, 1, &[9, 8, 7, 125], 2),
            Err(ScreenshotError::ReferenceMismatch)
        ));
    }
}
