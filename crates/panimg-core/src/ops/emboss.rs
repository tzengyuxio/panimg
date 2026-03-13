use crate::error::{PanimgError, Result};
use crate::ops::{Operation, OperationDescription};
use crate::schema::{CommandSchema, ParamSchema, ParamType};
use image::imageops::filter3x3;
use image::DynamicImage;

/// Emboss effect operation using a convolution kernel.
pub struct EmbossOp;

impl EmbossOp {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EmbossOp {
    fn default() -> Self {
        Self
    }
}

// Classic emboss kernel
const EMBOSS_KERNEL: [f32; 9] = [-2.0, -1.0, 0.0, -1.0, 1.0, 1.0, 0.0, 1.0, 2.0];

impl Operation<DynamicImage, PanimgError> for EmbossOp {
    fn name(&self) -> &str {
        "emboss"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        let rgba = img.to_rgba8();
        let filtered = filter3x3(&rgba, &EMBOSS_KERNEL);
        Ok(DynamicImage::ImageRgba8(filtered))
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "emboss".into(),
            params: serde_json::json!({}),
            description: "Apply emboss effect".into(),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "emboss".into(),
            description: "Apply an emboss effect to an image".into(),
            params: vec![
                ParamSchema {
                    name: "input".into(),
                    param_type: ParamType::Path,
                    required: true,
                    description: "Input image path".into(),
                    default: None,
                    choices: None,
                    range: None,
                },
                ParamSchema {
                    name: "output".into(),
                    param_type: ParamType::Path,
                    required: true,
                    description: "Output image path".into(),
                    default: None,
                    choices: None,
                    range: None,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_image() -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::from_fn(8, 8, |x, _y| {
            if x < 4 {
                image::Rgba([255, 255, 255, 255])
            } else {
                image::Rgba([0, 0, 0, 255])
            }
        }))
    }

    #[test]
    fn emboss_preserves_dimensions() {
        let op = EmbossOp::new();
        let result = op.apply(test_image()).unwrap();
        assert_eq!(result.width(), 8);
        assert_eq!(result.height(), 8);
    }

    #[test]
    fn emboss_produces_output() {
        let op = EmbossOp::new();
        let result = op.apply(test_image()).unwrap();
        let rgba = result.to_rgba8();
        // Emboss should produce non-uniform output from an image with edges
        let p1 = rgba.get_pixel(2, 4);
        let p2 = rgba.get_pixel(5, 4);
        // Left and right sides should differ
        assert_ne!(p1[0], p2[0]);
    }
}
