use crate::error::Result;
use crate::ops::{Operation, OperationDescription};
use crate::schema::{CommandSchema, ParamSchema, ParamType};
use image::imageops::filter3x3;
use image::DynamicImage;

/// Edge detection operation using a Laplacian kernel.
pub struct EdgeDetectOp;

impl EdgeDetectOp {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EdgeDetectOp {
    fn default() -> Self {
        Self
    }
}

// Laplacian kernel for edge detection
const LAPLACIAN_KERNEL: [f32; 9] = [0.0, -1.0, 0.0, -1.0, 4.0, -1.0, 0.0, -1.0, 0.0];

impl Operation for EdgeDetectOp {
    fn name(&self) -> &str {
        "edge-detect"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        // Convert to rgba8, apply filter, wrap back
        let rgba = img.to_rgba8();
        let filtered = filter3x3(&rgba, &LAPLACIAN_KERNEL);
        Ok(DynamicImage::ImageRgba8(filtered))
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "edge-detect".into(),
            params: serde_json::json!({}),
            description: "Detect edges using Laplacian kernel".into(),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "edge-detect".into(),
            description: "Detect edges in an image using a Laplacian kernel".into(),
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
        // Image with a sharp edge: left half white, right half black
        DynamicImage::ImageRgba8(image::RgbaImage::from_fn(8, 8, |x, _y| {
            if x < 4 {
                image::Rgba([255, 255, 255, 255])
            } else {
                image::Rgba([0, 0, 0, 255])
            }
        }))
    }

    #[test]
    fn edge_detect_preserves_dimensions() {
        let op = EdgeDetectOp::new();
        let result = op.apply(test_image()).unwrap();
        assert_eq!(result.width(), 8);
        assert_eq!(result.height(), 8);
    }

    #[test]
    fn edge_detect_finds_edges() {
        let op = EdgeDetectOp::new();
        let result = op.apply(test_image()).unwrap();
        let rgba = result.to_rgba8();
        // Interior pixels (away from edge) should be dark
        let interior = rgba.get_pixel(1, 4);
        // Edge pixels (at boundary x=3/4) should be brighter
        let edge = rgba.get_pixel(3, 4);
        assert!(edge[0] > interior[0]);
    }

    #[test]
    fn edge_detect_uniform_image_is_dark() {
        let uniform = DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            8,
            8,
            image::Rgba([128, 128, 128, 255]),
        ));
        let op = EdgeDetectOp::new();
        let result = op.apply(uniform).unwrap();
        let rgba = result.to_rgba8();
        // Center pixel should be near zero (no edges)
        let p = rgba.get_pixel(4, 4);
        assert!(p[0] < 10);
    }
}
