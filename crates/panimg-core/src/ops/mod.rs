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
pub mod resize;
pub mod rotate;
pub mod sharpen;
pub mod trim;

use crate::error::Result;
use crate::schema::CommandSchema;
use image::DynamicImage;
use serde::Serialize;

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
