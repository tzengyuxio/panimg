use crate::error::{PanimgError, Result};
use crate::ops::{Operation, OperationDescription};
use crate::schema::{CommandSchema, ParamSchema, ParamType};
use image::DynamicImage;

/// Invert (negate) image colors.
pub struct InvertOp;

impl Default for InvertOp {
    fn default() -> Self {
        Self
    }
}

impl InvertOp {
    pub fn new() -> Self {
        Self
    }
}

impl Operation<DynamicImage, PanimgError> for InvertOp {
    fn name(&self) -> &str {
        "invert"
    }

    fn apply(&self, mut img: DynamicImage) -> Result<DynamicImage> {
        img.invert();
        Ok(img)
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "invert".into(),
            params: serde_json::json!({}),
            description: "Invert colors".into(),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "invert".into(),
            description: "Invert (negate) image colors".into(),
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

    #[test]
    fn invert_preserves_dimensions() {
        let img = DynamicImage::ImageRgba8(image::RgbaImage::from_fn(4, 4, |_, _| {
            image::Rgba([100, 150, 200, 255])
        }));
        let op = InvertOp::new();
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 4);
        assert_eq!(result.height(), 4);
    }

    #[test]
    fn invert_negates_colors() {
        // Use Rgb (not Rgba) to avoid alpha inversion complications
        let img = DynamicImage::ImageRgb8(image::RgbImage::from_fn(4, 4, |_, _| {
            image::Rgb([100, 150, 200])
        }));
        let op = InvertOp::new();
        let result = op.apply(img).unwrap();
        let rgb = result.to_rgb8();
        let p = rgb.get_pixel(0, 0);
        assert_eq!(p[0], 155); // 255 - 100
        assert_eq!(p[1], 105); // 255 - 150
        assert_eq!(p[2], 55); // 255 - 200
    }

    #[test]
    fn invert_double_is_identity() {
        let original = DynamicImage::ImageRgb8(image::RgbImage::from_fn(4, 4, |_, _| {
            image::Rgb([100, 150, 200])
        }));
        let op = InvertOp::new();
        let once = op.apply(original.clone()).unwrap();
        let twice = op.apply(once).unwrap();
        let orig_rgba = original.to_rgb8();
        let result_rgba = twice.to_rgb8();
        assert_eq!(orig_rgba.get_pixel(0, 0), result_rgba.get_pixel(0, 0));
    }
}
