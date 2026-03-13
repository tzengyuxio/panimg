use crate::error::{PanimgError, Result};
use crate::ops::{Operation, OperationDescription};
use crate::schema::{CommandSchema, ParamRange, ParamSchema, ParamType};
use image::DynamicImage;

/// Rotate the hue of image colors.
pub struct HueRotateOp {
    /// Hue rotation in degrees (-360 to 360).
    pub degrees: i32,
}

impl HueRotateOp {
    pub fn new(degrees: i32) -> Result<Self> {
        if !(-360..=360).contains(&degrees) {
            return Err(PanimgError::InvalidArgument {
                message: format!("hue-rotate value {degrees} out of range"),
                suggestion: "use a value between -360 and 360".into(),
            });
        }
        Ok(Self { degrees })
    }
}

impl Operation<DynamicImage, PanimgError> for HueRotateOp {
    fn name(&self) -> &str {
        "hue-rotate"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        Ok(img.huerotate(self.degrees))
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "hue-rotate".into(),
            params: serde_json::json!({ "degrees": self.degrees }),
            description: format!("Rotate hue by {}°", self.degrees),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "hue-rotate".into(),
            description: "Rotate image hue".into(),
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
                    name: "degrees".into(),
                    param_type: ParamType::Integer,
                    required: true,
                    description: "Hue rotation in degrees (-360 to 360)".into(),
                    default: None,
                    choices: None,
                    range: Some(ParamRange {
                        min: -360.0,
                        max: 360.0,
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
            image::Rgba([255, 0, 0, 255]) // pure red
        }))
    }

    #[test]
    fn hue_rotate_preserves_dimensions() {
        let op = HueRotateOp::new(90).unwrap();
        let result = op.apply(test_image()).unwrap();
        assert_eq!(result.width(), 4);
        assert_eq!(result.height(), 4);
    }

    #[test]
    fn hue_rotate_changes_color() {
        let op = HueRotateOp::new(120).unwrap();
        let result = op.apply(test_image()).unwrap();
        let rgba = result.to_rgba8();
        let p = rgba.get_pixel(0, 0);
        // Red rotated 120° should shift towards green
        assert!(p[1] > p[0]); // green should dominate
    }

    #[test]
    fn hue_rotate_360_is_identity() {
        let op = HueRotateOp::new(360).unwrap();
        let img = test_image();
        let original = img.to_rgba8().get_pixel(0, 0).0;
        let result = op.apply(img).unwrap();
        let rgba = result.to_rgba8();
        let p = rgba.get_pixel(0, 0).0;
        // 360° rotation should be approximately identity
        assert!((p[0] as i32 - original[0] as i32).unsigned_abs() <= 1);
    }

    #[test]
    fn hue_rotate_out_of_range() {
        assert!(HueRotateOp::new(361).is_err());
        assert!(HueRotateOp::new(-361).is_err());
    }
}
