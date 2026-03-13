use crate::error::{PanimgError, Result};
use crate::ops::{Operation, OperationDescription};
use crate::schema::{CommandSchema, ParamRange, ParamSchema, ParamType};
use image::DynamicImage;

/// Gaussian blur operation.
pub struct BlurOp {
    /// Blur sigma (radius). Higher values = more blur.
    pub sigma: f32,
}

impl BlurOp {
    pub fn new(sigma: f32) -> Result<Self> {
        if sigma <= 0.0 {
            return Err(PanimgError::InvalidArgument {
                message: format!("blur sigma must be positive, got {sigma}"),
                suggestion: "use a value like 1.0, 2.5, or 5.0".into(),
            });
        }
        if sigma > 100.0 {
            return Err(PanimgError::InvalidArgument {
                message: format!("blur sigma {sigma} is too large"),
                suggestion: "use a value between 0.1 and 100.0".into(),
            });
        }
        Ok(Self { sigma })
    }
}

impl Operation<DynamicImage, PanimgError> for BlurOp {
    fn name(&self) -> &str {
        "blur"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        Ok(img.blur(self.sigma))
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "blur".into(),
            params: serde_json::json!({ "sigma": self.sigma }),
            description: format!("Gaussian blur (sigma={})", self.sigma),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "blur".into(),
            description: "Apply Gaussian blur to an image".into(),
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
                    description: "Blur radius (sigma). Higher values = more blur".into(),
                    default: None,
                    choices: None,
                    range: Some(ParamRange {
                        min: 0.1,
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
        DynamicImage::ImageRgba8(image::RgbaImage::from_fn(8, 8, |x, y| {
            // Checkerboard pattern — blur should smooth it
            if (x + y) % 2 == 0 {
                image::Rgba([255, 255, 255, 255])
            } else {
                image::Rgba([0, 0, 0, 255])
            }
        }))
    }

    #[test]
    fn blur_preserves_dimensions() {
        let op = BlurOp::new(1.0).unwrap();
        let result = op.apply(test_image()).unwrap();
        assert_eq!(result.width(), 8);
        assert_eq!(result.height(), 8);
    }

    #[test]
    fn blur_smooths_checkerboard() {
        let op = BlurOp::new(2.0).unwrap();
        let img = test_image();
        let result = op.apply(img).unwrap();
        let rgba = result.to_rgba8();
        // After blur, pixels should be somewhere between 0 and 255
        let p = rgba.get_pixel(4, 4);
        assert!(p[0] > 50 && p[0] < 200);
    }

    #[test]
    fn blur_invalid_sigma() {
        assert!(BlurOp::new(0.0).is_err());
        assert!(BlurOp::new(-1.0).is_err());
        assert!(BlurOp::new(101.0).is_err());
    }
}
