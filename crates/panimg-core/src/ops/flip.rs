use crate::error::{PanimgError, Result};
use crate::ops::{Operation, OperationDescription};
use crate::schema::{CommandSchema, ParamSchema, ParamType};
use image::DynamicImage;

/// Flip direction.
#[derive(Debug, Clone, Copy)]
pub enum FlipDirection {
    Horizontal,
    Vertical,
}

impl FlipDirection {
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "horizontal" | "h" | "x" => Ok(Self::Horizontal),
            "vertical" | "v" | "y" => Ok(Self::Vertical),
            _ => Err(PanimgError::InvalidArgument {
                message: format!("unknown flip direction: '{s}'"),
                suggestion: "use horizontal (h) or vertical (v)".into(),
            }),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

/// Flip (mirror) operation.
pub struct FlipOp {
    pub direction: FlipDirection,
}

impl FlipOp {
    pub fn new(direction: FlipDirection) -> Self {
        Self { direction }
    }
}

impl Operation for FlipOp {
    fn name(&self) -> &str {
        "flip"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        Ok(match self.direction {
            FlipDirection::Horizontal => img.fliph(),
            FlipDirection::Vertical => img.flipv(),
        })
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "flip".into(),
            params: serde_json::json!({ "direction": self.direction.as_str() }),
            description: format!("Flip {}", self.direction.as_str()),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "flip".into(),
            description: "Flip (mirror) an image horizontally or vertically".into(),
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
                    name: "direction".into(),
                    param_type: ParamType::String,
                    required: true,
                    description: "Flip direction".into(),
                    default: None,
                    choices: Some(vec!["horizontal".into(), "vertical".into()]),
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
        // Create a 4x4 image with distinct pixel values so we can verify flipping
        DynamicImage::ImageRgba8(image::RgbaImage::from_fn(4, 4, |x, y| {
            image::Rgba([x as u8 * 10, y as u8 * 10, 0, 255])
        }))
    }

    #[test]
    fn flip_horizontal_preserves_dimensions() {
        let img = test_image();
        let op = FlipOp::new(FlipDirection::Horizontal);
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 4);
        assert_eq!(result.height(), 4);
    }

    #[test]
    fn flip_vertical_preserves_dimensions() {
        let img = test_image();
        let op = FlipOp::new(FlipDirection::Vertical);
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 4);
        assert_eq!(result.height(), 4);
    }

    #[test]
    fn flip_horizontal_mirrors_pixels() {
        let img = test_image();
        let op = FlipOp::new(FlipDirection::Horizontal);
        let result = op.apply(img).unwrap();
        let rgba = result.to_rgba8();
        // First pixel of first row should now be what was the last pixel
        assert_eq!(rgba.get_pixel(0, 0)[0], 30); // was x=3 → 3*10=30
        assert_eq!(rgba.get_pixel(3, 0)[0], 0); // was x=0 → 0*10=0
    }

    #[test]
    fn parse_direction_aliases() {
        assert!(matches!(
            FlipDirection::parse("h"),
            Ok(FlipDirection::Horizontal)
        ));
        assert!(matches!(
            FlipDirection::parse("v"),
            Ok(FlipDirection::Vertical)
        ));
        assert!(matches!(
            FlipDirection::parse("x"),
            Ok(FlipDirection::Horizontal)
        ));
        assert!(FlipDirection::parse("diagonal").is_err());
    }
}
