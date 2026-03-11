use crate::error::Result;
use crate::ops::{Operation, OperationDescription};
use crate::schema::{CommandSchema, ParamSchema, ParamType};
use image::DynamicImage;

/// Convert image to grayscale.
pub struct GrayscaleOp;

impl Default for GrayscaleOp {
    fn default() -> Self {
        Self
    }
}

impl GrayscaleOp {
    pub fn new() -> Self {
        Self
    }
}

impl Operation for GrayscaleOp {
    fn name(&self) -> &str {
        "grayscale"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        Ok(img.grayscale())
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "grayscale".into(),
            params: serde_json::json!({}),
            description: "Convert to grayscale".into(),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "grayscale".into(),
            description: "Convert an image to grayscale".into(),
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
        DynamicImage::ImageRgba8(image::RgbaImage::from_fn(4, 4, |x, y| {
            image::Rgba([x as u8 * 50, y as u8 * 50, 100, 255])
        }))
    }

    #[test]
    fn grayscale_preserves_dimensions() {
        let img = test_image();
        let op = GrayscaleOp::new();
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 4);
        assert_eq!(result.height(), 4);
    }

    #[test]
    fn grayscale_produces_gray_pixels() {
        let img = DynamicImage::ImageRgba8(image::RgbaImage::from_fn(4, 4, |_, _| {
            image::Rgba([255, 0, 0, 255]) // pure red
        }));
        let op = GrayscaleOp::new();
        let result = op.apply(img).unwrap();
        let rgba = result.to_rgba8();
        let p = rgba.get_pixel(0, 0);
        // In grayscale, R == G == B
        assert_eq!(p[0], p[1]);
        assert_eq!(p[1], p[2]);
    }
}
