pub mod animation;
pub mod blur;
pub mod brightness;
pub mod color;
pub mod contrast;
pub mod crop;
pub mod diff;
pub mod draw;
pub mod edge_detect;
pub mod emboss;
pub mod flip;
pub mod grayscale;
pub mod hue_rotate;
pub mod invert;
pub mod orient;
pub mod overlay;
pub mod position;
pub mod resize;
pub mod rotate;
pub mod sharpen;
pub mod tilt_shift;
pub mod trim;

#[cfg(feature = "text")]
pub mod text;

use crate::error::Result;
use crate::schema::CommandSchema;
use image::{DynamicImage, Rgba};
use serde::Serialize;

/// Blend a color onto a pixel with alpha compositing (Porter-Duff "over").
///
/// `coverage` scales the source alpha (0.0–1.0), useful for sub-pixel glyph
/// rendering. Pass `1.0` for normal opaque blending.
pub(crate) fn blend_pixel(base: &Rgba<u8>, color: &Rgba<u8>, coverage: f32) -> Rgba<u8> {
    let ca = (color[3] as f32 / 255.0) * coverage;
    let ba = base[3] as f32 / 255.0;
    let out_a = ca + ba * (1.0 - ca);
    if out_a == 0.0 {
        return Rgba([0, 0, 0, 0]);
    }
    let blend = |cc: u8, bc: u8| -> u8 {
        let c = (cc as f32 / 255.0 * ca + bc as f32 / 255.0 * ba * (1.0 - ca)) / out_a;
        (c * 255.0).round().clamp(0.0, 255.0) as u8
    };
    Rgba([
        blend(color[0], base[0]),
        blend(color[1], base[1]),
        blend(color[2], base[2]),
        (out_a * 255.0).round().clamp(0.0, 255.0) as u8,
    ])
}

/// Description of an operation for dry-run output.
#[derive(Debug, Clone, Serialize)]
pub struct OperationDescription {
    pub operation: String,
    pub params: serde_json::Value,
    pub description: String,
}

/// Trait for image processing operations.
pub trait Operation: Send + Sync {
    /// Human-readable name.
    fn name(&self) -> &str;

    /// Apply the operation to an image.
    fn apply(&self, img: DynamicImage) -> Result<DynamicImage>;

    /// Describe what this operation will do (for dry-run).
    fn describe(&self) -> OperationDescription;

    /// Parameter schema for this operation.
    fn schema() -> CommandSchema
    where
        Self: Sized;
}
