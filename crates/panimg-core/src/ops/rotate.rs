use crate::error::{PanimgError, Result};
use crate::ops::{Operation, OperationDescription};
use crate::schema::{CommandSchema, ParamSchema, ParamType};
use image::DynamicImage;

/// Rotation angle (multiples of 90 degrees).
#[derive(Debug, Clone, Copy)]
pub enum RotateAngle {
    Deg90,
    Deg180,
    Deg270,
}

impl RotateAngle {
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim() {
            "90" | "90cw" | "right" => Ok(Self::Deg90),
            "180" => Ok(Self::Deg180),
            "270" | "90ccw" | "left" => Ok(Self::Deg270),
            _ => Err(PanimgError::InvalidArgument {
                message: format!("unsupported rotation angle: '{s}'"),
                suggestion: "use 90, 180, 270, left, or right".into(),
            }),
        }
    }

    fn degrees(self) -> u32 {
        match self {
            Self::Deg90 => 90,
            Self::Deg180 => 180,
            Self::Deg270 => 270,
        }
    }
}

/// Rotate operation (90/180/270 degrees).
pub struct RotateOp {
    pub angle: RotateAngle,
}

impl RotateOp {
    pub fn new(angle: RotateAngle) -> Self {
        Self { angle }
    }
}

impl Operation for RotateOp {
    fn name(&self) -> &str {
        "rotate"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        Ok(match self.angle {
            RotateAngle::Deg90 => img.rotate90(),
            RotateAngle::Deg180 => img.rotate180(),
            RotateAngle::Deg270 => img.rotate270(),
        })
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "rotate".into(),
            params: serde_json::json!({ "angle": self.angle.degrees() }),
            description: format!("Rotate {} degrees clockwise", self.angle.degrees()),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "rotate".into(),
            description: "Rotate an image by 90, 180, or 270 degrees".into(),
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
                    name: "angle".into(),
                    param_type: ParamType::String,
                    required: true,
                    description: "Rotation angle".into(),
                    default: None,
                    choices: Some(vec![
                        "90".into(),
                        "180".into(),
                        "270".into(),
                        "left".into(),
                        "right".into(),
                    ]),
                    range: None,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_image(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::from_fn(w, h, |_, _| {
            image::Rgba([128, 128, 128, 255])
        }))
    }

    #[test]
    fn rotate_90_swaps_dimensions() {
        let img = test_image(100, 50);
        let op = RotateOp::new(RotateAngle::Deg90);
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 50);
        assert_eq!(result.height(), 100);
    }

    #[test]
    fn rotate_180_preserves_dimensions() {
        let img = test_image(100, 50);
        let op = RotateOp::new(RotateAngle::Deg180);
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 100);
        assert_eq!(result.height(), 50);
    }

    #[test]
    fn rotate_270_swaps_dimensions() {
        let img = test_image(100, 50);
        let op = RotateOp::new(RotateAngle::Deg270);
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 50);
        assert_eq!(result.height(), 100);
    }

    #[test]
    fn parse_angle_aliases() {
        assert!(matches!(
            RotateAngle::parse("right"),
            Ok(RotateAngle::Deg90)
        ));
        assert!(matches!(
            RotateAngle::parse("left"),
            Ok(RotateAngle::Deg270)
        ));
        assert!(matches!(
            RotateAngle::parse("90ccw"),
            Ok(RotateAngle::Deg270)
        ));
        assert!(RotateAngle::parse("45").is_err());
    }
}
