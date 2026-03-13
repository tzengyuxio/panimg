use crate::error::{PanimgError, Result};
use crate::ops::{Operation, OperationDescription};
use crate::schema::{CommandSchema, ParamRange, ParamSchema, ParamType};
use image::DynamicImage;

/// Adjust image contrast.
pub struct ContrastOp {
    /// Contrast adjustment value (-100 to 100).
    /// Positive values increase contrast, negative values decrease it.
    pub value: f32,
}

impl ContrastOp {
    pub fn new(value: f32) -> Result<Self> {
        if !(-100.0..=100.0).contains(&value) {
            return Err(PanimgError::InvalidArgument {
                message: format!("contrast value {value} out of range"),
                suggestion: "use a value between -100 and 100".into(),
            });
        }
        Ok(Self { value })
    }
}

impl Operation<DynamicImage, PanimgError> for ContrastOp {
    fn name(&self) -> &str {
        "contrast"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        Ok(img.adjust_contrast(self.value))
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "contrast".into(),
            params: serde_json::json!({ "value": self.value }),
            description: format!("Adjust contrast by {}", self.value),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "contrast".into(),
            description: "Adjust image contrast".into(),
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
                ParamSchema {
                    name: "value".into(),
                    param_type: ParamType::Float,
                    required: true,
                    description: "Contrast adjustment (-100 to 100)".into(),
                    default: None,
                    choices: None,
                    range: Some(ParamRange {
                        min: -100.0,
                        max: 100.0,
                    }),
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_image() -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::from_fn(4, 4, |x, _| {
            image::Rgba([x as u8 * 80, x as u8 * 80, x as u8 * 80, 255])
        }))
    }

    #[test]
    fn contrast_preserves_dimensions() {
        let op = ContrastOp::new(30.0).unwrap();
        let result = op.apply(test_image()).unwrap();
        assert_eq!(result.width(), 4);
        assert_eq!(result.height(), 4);
    }

    #[test]
    fn contrast_zero_minimal_change() {
        let op = ContrastOp::new(0.0).unwrap();
        let img = test_image();
        let original = img.to_rgba8().get_pixel(1, 0).0;
        let result = op.apply(img).unwrap();
        let rgba = result.to_rgba8();
        let p = rgba.get_pixel(1, 0).0;
        assert_eq!(p[0], original[0]);
    }

    #[test]
    fn contrast_out_of_range() {
        assert!(ContrastOp::new(101.0).is_err());
        assert!(ContrastOp::new(-101.0).is_err());
    }
}
