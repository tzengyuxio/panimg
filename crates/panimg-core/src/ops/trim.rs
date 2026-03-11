use crate::error::Result;
use crate::ops::{Operation, OperationDescription};
use crate::schema::{CommandSchema, ParamRange, ParamSchema, ParamType};
use image::DynamicImage;

/// Trim (auto-crop) whitespace or similar-colored borders from an image.
///
/// Compares border pixels against a reference color (top-left pixel by default)
/// and crops to the bounding box of non-matching content.
pub struct TrimOp {
    /// Color distance threshold (0-255). Pixels within this distance of the
    /// reference color are considered "background" and trimmed.
    pub tolerance: u8,
}

impl TrimOp {
    pub fn new(tolerance: u8) -> Result<Self> {
        Ok(Self { tolerance })
    }
}

impl Operation for TrimOp {
    fn name(&self) -> &str {
        "trim"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        let rgba = img.to_rgba8();
        let (w, h) = (rgba.width(), rgba.height());

        if w == 0 || h == 0 {
            return Ok(img);
        }

        // Reference color: top-left pixel
        let ref_pixel = rgba.get_pixel(0, 0);
        let tol = self.tolerance as i32;

        let is_bg = |x: u32, y: u32| -> bool {
            let p = rgba.get_pixel(x, y);
            (p[0] as i32 - ref_pixel[0] as i32).abs() <= tol
                && (p[1] as i32 - ref_pixel[1] as i32).abs() <= tol
                && (p[2] as i32 - ref_pixel[2] as i32).abs() <= tol
                && (p[3] as i32 - ref_pixel[3] as i32).abs() <= tol
        };

        // Find bounding box of non-background content
        let mut min_x = w;
        let mut min_y = h;
        let mut max_x = 0u32;
        let mut max_y = 0u32;

        for y in 0..h {
            for x in 0..w {
                if !is_bg(x, y) {
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                }
            }
        }

        // If entire image is background, return as-is
        if max_x < min_x || max_y < min_y {
            return Ok(img);
        }

        let crop_w = max_x - min_x + 1;
        let crop_h = max_y - min_y + 1;

        // If no trimming needed
        if min_x == 0 && min_y == 0 && crop_w == w && crop_h == h {
            return Ok(img);
        }

        let cropped = image::imageops::crop_imm(&rgba, min_x, min_y, crop_w, crop_h).to_image();
        Ok(DynamicImage::ImageRgba8(cropped))
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "trim".into(),
            params: serde_json::json!({ "tolerance": self.tolerance }),
            description: format!("Trim whitespace/border (tolerance={})", self.tolerance),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "trim".into(),
            description: "Trim (auto-crop) whitespace or similar-colored borders from an image"
                .into(),
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
                    name: "tolerance".into(),
                    param_type: ParamType::Integer,
                    required: false,
                    description: "Color distance threshold for background detection (0-255)".into(),
                    default: Some(serde_json::json!(10)),
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

    #[test]
    fn trim_white_border() {
        // 8x8 image: white border, red 4x4 center
        let img = DynamicImage::ImageRgba8(image::RgbaImage::from_fn(8, 8, |x, y| {
            if x >= 2 && x < 6 && y >= 2 && y < 6 {
                image::Rgba([255, 0, 0, 255])
            } else {
                image::Rgba([255, 255, 255, 255])
            }
        }));

        let op = TrimOp::new(0).unwrap();
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 4);
        assert_eq!(result.height(), 4);
    }

    #[test]
    fn trim_no_border() {
        // All red, no border to trim — top-left is red, everything matches
        let img = DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            8,
            8,
            image::Rgba([255, 0, 0, 255]),
        ));

        let op = TrimOp::new(0).unwrap();
        let result = op.apply(img).unwrap();
        // Entire image is "background" (matches top-left), returns as-is
        assert_eq!(result.width(), 8);
        assert_eq!(result.height(), 8);
    }

    #[test]
    fn trim_with_tolerance() {
        // 8x8: near-white border (250,250,250), red center
        let img = DynamicImage::ImageRgba8(image::RgbaImage::from_fn(8, 8, |x, y| {
            if x >= 2 && x < 6 && y >= 2 && y < 6 {
                image::Rgba([255, 0, 0, 255])
            } else {
                image::Rgba([250, 250, 250, 255])
            }
        }));

        // With 0 tolerance, border differs slightly from corner → might not trim
        // With 10 tolerance, those near-white pixels match the corner
        let op = TrimOp::new(10).unwrap();
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 4);
        assert_eq!(result.height(), 4);
    }

    #[test]
    fn trim_asymmetric_border() {
        // 8x6: white everywhere except a 2x2 block at (1,1)
        let img = DynamicImage::ImageRgba8(image::RgbaImage::from_fn(8, 6, |x, y| {
            if x >= 1 && x < 3 && y >= 1 && y < 3 {
                image::Rgba([0, 0, 0, 255])
            } else {
                image::Rgba([255, 255, 255, 255])
            }
        }));

        let op = TrimOp::new(0).unwrap();
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 2);
        assert_eq!(result.height(), 2);
    }

    #[test]
    fn trim_preserves_content() {
        let img = DynamicImage::ImageRgba8(image::RgbaImage::from_fn(6, 6, |x, y| {
            if x >= 1 && x < 5 && y >= 1 && y < 5 {
                image::Rgba([(x * 50) as u8, (y * 50) as u8, 128, 255])
            } else {
                image::Rgba([255, 255, 255, 255])
            }
        }));

        let op = TrimOp::new(0).unwrap();
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 4);
        assert_eq!(result.height(), 4);

        // Verify content is preserved
        let rgba = result.to_rgba8();
        let p = rgba.get_pixel(0, 0); // was (1,1) in original
        assert_eq!(p[0], 50);
        assert_eq!(p[1], 50);
    }
}
