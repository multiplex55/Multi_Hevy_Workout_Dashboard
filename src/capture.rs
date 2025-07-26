use egui::{ColorImage, Rect};
use image::{ImageBuffer, Rgba};
use std::path::Path;

/// Crop a [`ColorImage`] to the given `rect` in points.
pub fn crop_image(image: &ColorImage, rect: Rect, pixels_per_point: f32) -> ColorImage {
    let [w, h] = image.size;
    let x0 = (rect.min.x * pixels_per_point).round().clamp(0.0, w as f32) as usize;
    let y0 = (rect.min.y * pixels_per_point).round().clamp(0.0, h as f32) as usize;
    let x1 = (rect.max.x * pixels_per_point).round().clamp(0.0, w as f32) as usize;
    let y1 = (rect.max.y * pixels_per_point).round().clamp(0.0, h as f32) as usize;
    let width = x1.saturating_sub(x0);
    let height = y1.saturating_sub(y0);
    let mut pixels = Vec::with_capacity(width * height);
    for row in y0..y1 {
        let start = row * w + x0;
        let end = start + width;
        pixels.extend_from_slice(&image.pixels[start..end]);
    }
    ColorImage {
        size: [width, height],
        pixels,
    }
}

/// Save a [`ColorImage`] as a PNG file.
pub fn save_png(image: &ColorImage, path: &Path) -> Result<(), image::ImageError> {
    let mut bytes = Vec::with_capacity(image.pixels.len() * 4);
    for p in &image.pixels {
        bytes.extend_from_slice(&p.to_array());
    }
    let Some(img) =
        ImageBuffer::<Rgba<u8>, _>::from_vec(image.size[0] as u32, image.size[1] as u32, bytes)
    else {
        return Err(image::ImageError::Parameter(
            image::error::ParameterError::from_kind(
                image::error::ParameterErrorKind::DimensionMismatch,
            ),
        ));
    };
    img.save(path)
}
