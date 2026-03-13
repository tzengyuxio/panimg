use crate::error::PanimgError;
use image::DynamicImage;

pub use pan_common::pipeline::PipelinePlan;

/// Image processing pipeline — a type alias for the generic pipeline
/// specialized with `DynamicImage` and `PanimgError`.
pub type Pipeline = pan_common::pipeline::Pipeline<DynamicImage, PanimgError>;
