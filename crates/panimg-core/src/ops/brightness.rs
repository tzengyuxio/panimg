use crate::error::{PanimgError, Result};
use crate::ops::{Operation, OperationDescription};
use crate::schema::{CommandSchema, ParamRange, ParamSchema, ParamType};
use image::DynamicImage;

/// Adjust image brightness.
pub struct BrightnessOp {
    /// Brightness adjustment value (-100 to 100).
    /// Positive values brighten, negative values darken.
    pub value: i32,
}

impl BrightnessOp {
    pub fn new(value: i32) -> Result<Self> {
        if !(-100..=100).contains(&value) {
            return Err(PanimgError::InvalidArgument {
                message: format!("brightness value {value} out of range"),
                suggestion: "use a value between -100 and 100".into(),
            });
        }
        Ok(Self { value })
    }
}

impl Operation for BrightnessOp {
    fn name(&self) -> &str {
        "brightness"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        Ok(img.brighten(self.value))
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "brightness".into(),
            params: serde_json::json!({ "value": self.value }),
            description: format!("Adjust brightness by {}", self.value),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "brightness".into(),
            description: "Adjust image brightness".into(),
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
                    param_type: ParamType::Integer,
                    required: true,
                    description: "Brightness adjustment (-100 to 100)".into(),
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
        DynamicImage::ImageRgba8(image::RgbaImage::from_fn(4, 4, |_, _| {
            image::Rgba([100, 100, 100, 255])
        }))
    }

    #[test]
    fn brightness_increase() {
        let op = BrightnessOp::new(50).unwrap();
        let result = op.apply(test_image()).unwrap();
        let rgba = result.to_rgba8();
        let p = rgba.get_pixel(0, 0);
        assert!(p[0] > 100);
    }

    #[test]
    fn brightness_decrease() {
        let op = BrightnessOp::new(-50).unwrap();
        let result = op.apply(test_image()).unwrap();
        let rgba = result.to_rgba8();
        let p = rgba.get_pixel(0, 0);
        assert!(p[0] < 100);
    }

    #[test]
    fn brightness_zero_no_change() {
        let op = BrightnessOp::new(0).unwrap();
        let result = op.apply(test_image()).unwrap();
        let rgba = result.to_rgba8();
        let p = rgba.get_pixel(0, 0);
        assert_eq!(p[0], 100);
    }

    #[test]
    fn brightness_out_of_range() {
        assert!(BrightnessOp::new(101).is_err());
        assert!(BrightnessOp::new(-101).is_err());
    }
}
