use crate::error::Result;
use crate::ops::{Operation, OperationDescription};
use crate::schema::{CommandSchema, ParamSchema, ParamType};
use image::DynamicImage;

/// Auto-orient based on EXIF orientation tag.
/// Applies the transformation indicated by the EXIF orientation value,
/// then the image should be treated as orientation=1 (normal).
pub struct AutoOrientOp {
    orientation: u32,
}

impl AutoOrientOp {
    /// Create from an EXIF orientation value (1-8).
    /// If orientation is 1 or unknown, this is a no-op.
    pub fn new(orientation: u32) -> Self {
        Self { orientation }
    }

    /// Read EXIF orientation from a file path.
    pub fn from_path(path: &std::path::Path) -> Self {
        let orientation = read_exif_orientation(path).unwrap_or(1);
        Self::new(orientation)
    }
}

fn read_exif_orientation(path: &std::path::Path) -> Option<u32> {
    let file = std::fs::File::open(path).ok()?;
    let mut reader = std::io::BufReader::new(file);
    let exif = exif::Reader::new().read_from_container(&mut reader).ok()?;
    let field = exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)?;
    field.value.get_uint(0)
}

impl Operation for AutoOrientOp {
    fn name(&self) -> &str {
        "auto-orient"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        // EXIF orientation values:
        // 1: normal
        // 2: flip horizontal
        // 3: rotate 180
        // 4: flip vertical
        // 5: transpose (rotate 90 CW + flip horizontal)
        // 6: rotate 90 CW
        // 7: transverse (rotate 90 CCW + flip horizontal)
        // 8: rotate 90 CCW
        Ok(match self.orientation {
            1 => img,
            2 => img.fliph(),
            3 => img.rotate180(),
            4 => img.flipv(),
            5 => img.rotate90().fliph(),
            6 => img.rotate90(),
            7 => img.rotate270().fliph(),
            8 => img.rotate270(),
            _ => img, // Unknown orientation, treat as normal
        })
    }

    fn describe(&self) -> OperationDescription {
        let desc = match self.orientation {
            1 => "No rotation needed (orientation=1)".into(),
            2 => "Flip horizontal".into(),
            3 => "Rotate 180 degrees".into(),
            4 => "Flip vertical".into(),
            5 => "Transpose (rotate 90 CW + flip horizontal)".into(),
            6 => "Rotate 90 degrees CW".into(),
            7 => "Transverse (rotate 90 CCW + flip horizontal)".into(),
            8 => "Rotate 90 degrees CCW".into(),
            _ => format!("Unknown orientation {}", self.orientation),
        };
        OperationDescription {
            operation: "auto-orient".into(),
            params: serde_json::json!({ "exif_orientation": self.orientation }),
            description: desc,
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "auto-orient".into(),
            description: "Auto-rotate image based on EXIF orientation tag".into(),
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

    fn test_image(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::from_fn(w, h, |_, _| {
            image::Rgba([128, 128, 128, 255])
        }))
    }

    #[test]
    fn orientation_1_noop() {
        let img = test_image(100, 50);
        let op = AutoOrientOp::new(1);
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 100);
        assert_eq!(result.height(), 50);
    }

    #[test]
    fn orientation_6_rotates_90() {
        let img = test_image(100, 50);
        let op = AutoOrientOp::new(6);
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 50);
        assert_eq!(result.height(), 100);
    }

    #[test]
    fn orientation_3_rotates_180() {
        let img = test_image(100, 50);
        let op = AutoOrientOp::new(3);
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 100);
        assert_eq!(result.height(), 50);
    }

    #[test]
    fn orientation_8_rotates_270() {
        let img = test_image(100, 50);
        let op = AutoOrientOp::new(8);
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 50);
        assert_eq!(result.height(), 100);
    }

    #[test]
    fn unknown_orientation_noop() {
        let img = test_image(100, 50);
        let op = AutoOrientOp::new(99);
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 100);
        assert_eq!(result.height(), 50);
    }
}
