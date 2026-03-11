use crate::error::{PanimgError, Result};
use crate::ops::{Operation, OperationDescription};
use crate::schema::{CommandSchema, ParamRange, ParamSchema, ParamType};
use image::DynamicImage;

/// Crop operation: extract a rectangular region from an image.
pub struct CropOp {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl CropOp {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(PanimgError::InvalidArgument {
                message: "crop width and height must be greater than 0".into(),
                suggestion: "specify positive --width and --height values".into(),
            });
        }
        Ok(Self {
            x,
            y,
            width,
            height,
        })
    }

    /// Validate that the crop region fits within the source image.
    fn validate(&self, src_w: u32, src_h: u32) -> Result<()> {
        if self.x + self.width > src_w || self.y + self.height > src_h {
            return Err(PanimgError::InvalidArgument {
                message: format!(
                    "crop region {}x{}+{}+{} exceeds image bounds {}x{}",
                    self.width, self.height, self.x, self.y, src_w, src_h
                ),
                suggestion: "adjust crop coordinates to fit within the image".into(),
            });
        }
        Ok(())
    }
}

impl Operation for CropOp {
    fn name(&self) -> &str {
        "crop"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        self.validate(img.width(), img.height())?;
        Ok(img.crop_imm(self.x, self.y, self.width, self.height))
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "crop".into(),
            params: serde_json::json!({
                "x": self.x,
                "y": self.y,
                "width": self.width,
                "height": self.height,
            }),
            description: format!(
                "Crop {}x{} region at ({}, {})",
                self.width, self.height, self.x, self.y
            ),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "crop".into(),
            description: "Crop a rectangular region from an image".into(),
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
                    name: "x".into(),
                    param_type: ParamType::Integer,
                    required: false,
                    description: "Left offset in pixels".into(),
                    default: Some(serde_json::json!(0)),
                    choices: None,
                    range: Some(ParamRange {
                        min: 0.0,
                        max: 65535.0,
                    }),
                },
                ParamSchema {
                    name: "y".into(),
                    param_type: ParamType::Integer,
                    required: false,
                    description: "Top offset in pixels".into(),
                    default: Some(serde_json::json!(0)),
                    choices: None,
                    range: Some(ParamRange {
                        min: 0.0,
                        max: 65535.0,
                    }),
                },
                ParamSchema {
                    name: "width".into(),
                    param_type: ParamType::Integer,
                    required: true,
                    description: "Crop width in pixels".into(),
                    default: None,
                    choices: None,
                    range: Some(ParamRange {
                        min: 1.0,
                        max: 65535.0,
                    }),
                },
                ParamSchema {
                    name: "height".into(),
                    param_type: ParamType::Integer,
                    required: true,
                    description: "Crop height in pixels".into(),
                    default: None,
                    choices: None,
                    range: Some(ParamRange {
                        min: 1.0,
                        max: 65535.0,
                    }),
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_image(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::from_fn(w, h, |x, y| {
            image::Rgba([x as u8, y as u8, 128, 255])
        }))
    }

    #[test]
    fn crop_basic() {
        let img = test_image(100, 100);
        let op = CropOp::new(10, 10, 50, 50).unwrap();
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 50);
        assert_eq!(result.height(), 50);
    }

    #[test]
    fn crop_full_image() {
        let img = test_image(100, 100);
        let op = CropOp::new(0, 0, 100, 100).unwrap();
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 100);
        assert_eq!(result.height(), 100);
    }

    #[test]
    fn crop_out_of_bounds() {
        let img = test_image(100, 100);
        let op = CropOp::new(50, 50, 60, 60).unwrap();
        assert!(op.apply(img).is_err());
    }

    #[test]
    fn crop_zero_size_rejected() {
        assert!(CropOp::new(0, 0, 0, 50).is_err());
        assert!(CropOp::new(0, 0, 50, 0).is_err());
    }
}
