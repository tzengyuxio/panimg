use crate::error::{PanimgError, Result};
use crate::ops::{Operation, OperationDescription};
use crate::schema::{CommandSchema, ParamRange, ParamSchema, ParamType};
use image::DynamicImage;

/// Unsharp mask sharpening operation.
pub struct SharpenOp {
    /// Blur sigma for the unsharp mask. Controls the radius of the effect.
    pub sigma: f32,
    /// Threshold for applying the sharpening.
    /// Only differences above this threshold are sharpened.
    pub threshold: i32,
}

impl SharpenOp {
    pub fn new(sigma: f32, threshold: i32) -> Result<Self> {
        if sigma <= 0.0 {
            return Err(PanimgError::InvalidArgument {
                message: format!("sharpen sigma must be positive, got {sigma}"),
                suggestion: "use a value like 1.0 or 3.0".into(),
            });
        }
        if sigma > 100.0 {
            return Err(PanimgError::InvalidArgument {
                message: format!("sharpen sigma {sigma} is too large"),
                suggestion: "use a value between 0.1 and 100.0".into(),
            });
        }
        if threshold < 0 {
            return Err(PanimgError::InvalidArgument {
                message: format!("sharpen threshold must be non-negative, got {threshold}"),
                suggestion: "use a value like 0, 5, or 10".into(),
            });
        }
        Ok(Self { sigma, threshold })
    }
}

impl Operation<DynamicImage, PanimgError> for SharpenOp {
    fn name(&self) -> &str {
        "sharpen"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        Ok(img.unsharpen(self.sigma, self.threshold))
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "sharpen".into(),
            params: serde_json::json!({
                "sigma": self.sigma,
                "threshold": self.threshold,
            }),
            description: format!(
                "Unsharp mask (sigma={}, threshold={})",
                self.sigma, self.threshold
            ),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "sharpen".into(),
            description: "Sharpen an image using unsharp mask".into(),
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
                    name: "sigma".into(),
                    param_type: ParamType::Float,
                    required: true,
                    description: "Blur sigma for unsharp mask. Controls effect radius".into(),
                    default: None,
                    choices: None,
                    range: Some(ParamRange {
                        min: 0.1,
                        max: 100.0,
                    }),
                },
                ParamSchema {
                    name: "threshold".into(),
                    param_type: ParamType::Integer,
                    required: false,
                    description:
                        "Only sharpen differences above this threshold (0 = sharpen everything)"
                            .into(),
                    default: Some(serde_json::json!(0)),
                    choices: None,
                    range: Some(ParamRange {
                        min: 0.0,
                        max: 255.0,
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
        DynamicImage::ImageRgba8(image::RgbaImage::from_fn(8, 8, |x, _| {
            // Gradient — sharpening should enhance edges
            image::Rgba([(x * 32) as u8, (x * 32) as u8, (x * 32) as u8, 255])
        }))
    }

    #[test]
    fn sharpen_preserves_dimensions() {
        let op = SharpenOp::new(1.0, 0).unwrap();
        let result = op.apply(test_image()).unwrap();
        assert_eq!(result.width(), 8);
        assert_eq!(result.height(), 8);
    }

    #[test]
    fn sharpen_modifies_pixels() {
        let op = SharpenOp::new(3.0, 0).unwrap();
        let img = test_image();
        let original = img.to_rgba8().get_pixel(4, 4).0;
        let result = op.apply(img).unwrap();
        let rgba = result.to_rgba8();
        let sharpened = rgba.get_pixel(4, 4).0;
        // Sharpening should change pixel values
        assert_ne!(original, sharpened);
    }

    #[test]
    fn sharpen_invalid_sigma() {
        assert!(SharpenOp::new(0.0, 0).is_err());
        assert!(SharpenOp::new(-1.0, 0).is_err());
    }

    #[test]
    fn sharpen_invalid_threshold() {
        assert!(SharpenOp::new(1.0, -1).is_err());
    }
}
