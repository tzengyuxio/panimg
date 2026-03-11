use crate::error::{PanimgError, Result};
use crate::ops::{Operation, OperationDescription};
use crate::schema::{CommandSchema, ParamRange, ParamSchema, ParamType};
use image::{DynamicImage, RgbaImage};

/// Overlay (composite) one image on top of another.
pub struct OverlayOp {
    /// The image to overlay on top of the base.
    overlay: DynamicImage,
    /// X offset from left edge.
    pub x: i64,
    /// Y offset from top edge.
    pub y: i64,
    /// Opacity of the overlay (0.0 = transparent, 1.0 = fully opaque).
    pub opacity: f32,
}

impl OverlayOp {
    pub fn new(overlay: DynamicImage, x: i64, y: i64, opacity: f32) -> Result<Self> {
        if !(0.0..=1.0).contains(&opacity) {
            return Err(PanimgError::InvalidArgument {
                message: format!("opacity must be between 0.0 and 1.0, got {opacity}"),
                suggestion: "use a value like 0.5 for 50% opacity".into(),
            });
        }
        Ok(Self {
            overlay,
            x,
            y,
            opacity,
        })
    }
}

impl Operation for OverlayOp {
    fn name(&self) -> &str {
        "overlay"
    }

    fn apply(&self, base: DynamicImage) -> Result<DynamicImage> {
        let base_rgba = base.to_rgba8();
        let overlay_rgba = self.overlay.to_rgba8();

        let (base_w, base_h) = base_rgba.dimensions();
        let (overlay_w, overlay_h) = overlay_rgba.dimensions();

        let mut result = base_rgba;

        // Composite pixel by pixel, handling offset and opacity
        for oy in 0..overlay_h {
            let by = self.y + oy as i64;
            if by < 0 || by >= base_h as i64 {
                continue;
            }
            for ox in 0..overlay_w {
                let bx = self.x + ox as i64;
                if bx < 0 || bx >= base_w as i64 {
                    continue;
                }

                let overlay_pixel = overlay_rgba.get_pixel(ox, oy);
                let base_pixel = result.get_pixel(bx as u32, by as u32);

                // Apply opacity to overlay alpha
                let oa = (overlay_pixel[3] as f32 / 255.0) * self.opacity;
                let ba = base_pixel[3] as f32 / 255.0;

                // Alpha compositing (Porter-Duff "over" operator)
                let out_a = oa + ba * (1.0 - oa);
                if out_a == 0.0 {
                    continue;
                }

                let blend = |oc: u8, bc: u8| -> u8 {
                    let o = oc as f32 / 255.0;
                    let b = bc as f32 / 255.0;
                    let c = (o * oa + b * ba * (1.0 - oa)) / out_a;
                    (c * 255.0).round().clamp(0.0, 255.0) as u8
                };

                let r = blend(overlay_pixel[0], base_pixel[0]);
                let g = blend(overlay_pixel[1], base_pixel[1]);
                let b = blend(overlay_pixel[2], base_pixel[2]);
                let a = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;

                result.put_pixel(bx as u32, by as u32, image::Rgba([r, g, b, a]));
            }
        }

        Ok(DynamicImage::ImageRgba8(result))
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "overlay".into(),
            params: serde_json::json!({
                "x": self.x,
                "y": self.y,
                "opacity": self.opacity,
            }),
            description: format!(
                "Overlay image at ({}, {}) with opacity {}",
                self.x, self.y, self.opacity
            ),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "overlay".into(),
            description: "Overlay (composite) one image on top of another".into(),
            params: vec![
                ParamSchema {
                    name: "input".into(),
                    param_type: ParamType::Path,
                    required: true,
                    description: "Base image path".into(),
                    default: None,
                    choices: None,
                    range: None,
                },
                ParamSchema {
                    name: "layer".into(),
                    param_type: ParamType::Path,
                    required: true,
                    description: "Overlay image path".into(),
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
                    description: "X offset from left edge (can be negative)".into(),
                    default: Some(serde_json::json!(0)),
                    choices: None,
                    range: None,
                },
                ParamSchema {
                    name: "y".into(),
                    param_type: ParamType::Integer,
                    required: false,
                    description: "Y offset from top edge (can be negative)".into(),
                    default: Some(serde_json::json!(0)),
                    choices: None,
                    range: None,
                },
                ParamSchema {
                    name: "opacity".into(),
                    param_type: ParamType::Float,
                    required: false,
                    description: "Opacity of the overlay (0.0 = transparent, 1.0 = opaque)".into(),
                    default: Some(serde_json::json!(1.0)),
                    choices: None,
                    range: Some(ParamRange {
                        min: 0.0,
                        max: 1.0,
                    }),
                },
                ParamSchema {
                    name: "position".into(),
                    param_type: ParamType::String,
                    required: false,
                    description: "Named position (overrides x/y): center, top-left, top-right, bottom-left, bottom-right".into(),
                    default: None,
                    choices: Some(vec![
                        "center".into(),
                        "top-left".into(),
                        "top-right".into(),
                        "bottom-left".into(),
                        "bottom-right".into(),
                    ]),
                    range: None,
                },
            ],
        }
    }
}

/// Calculate x/y offset for named positions.
pub fn resolve_position(
    position: &str,
    base_w: u32,
    base_h: u32,
    overlay_w: u32,
    overlay_h: u32,
    margin: i64,
) -> Result<(i64, i64)> {
    match position {
        "center" => Ok((
            (base_w as i64 - overlay_w as i64) / 2,
            (base_h as i64 - overlay_h as i64) / 2,
        )),
        "top-left" => Ok((margin, margin)),
        "top-right" => Ok((base_w as i64 - overlay_w as i64 - margin, margin)),
        "bottom-left" => Ok((margin, base_h as i64 - overlay_h as i64 - margin)),
        "bottom-right" => Ok((
            base_w as i64 - overlay_w as i64 - margin,
            base_h as i64 - overlay_h as i64 - margin,
        )),
        _ => Err(PanimgError::InvalidArgument {
            message: format!("unknown position: '{position}'"),
            suggestion: "use: center, top-left, top-right, bottom-left, bottom-right".into(),
        }),
    }
}

/// Create an OverlayOp that tiles the overlay image across the entire base image.
pub fn create_tiled_overlay(
    overlay: &DynamicImage,
    base_w: u32,
    base_h: u32,
    opacity: f32,
    spacing: u32,
) -> Result<DynamicImage> {
    let overlay_rgba = overlay.to_rgba8();
    let (ow, oh) = overlay_rgba.dimensions();

    let mut tiled = RgbaImage::new(base_w, base_h);

    let step_x = ow + spacing;
    let step_y = oh + spacing;

    let mut ty = 0u32;
    while ty < base_h {
        let mut tx = 0u32;
        while tx < base_w {
            for py in 0..oh {
                let dy = ty + py;
                if dy >= base_h {
                    break;
                }
                for px in 0..ow {
                    let dx = tx + px;
                    if dx >= base_w {
                        break;
                    }
                    tiled.put_pixel(dx, dy, *overlay_rgba.get_pixel(px, py));
                }
            }
            tx += step_x;
        }
        ty += step_y;
    }

    // The tiled image is used as overlay, opacity applied in OverlayOp
    let _ = opacity; // opacity is applied by the caller via OverlayOp
    Ok(DynamicImage::ImageRgba8(tiled))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn red_image(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            w,
            h,
            image::Rgba([255, 0, 0, 255]),
        ))
    }

    fn blue_image(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            w,
            h,
            image::Rgba([0, 0, 255, 255]),
        ))
    }

    fn semi_transparent_green(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            w,
            h,
            image::Rgba([0, 255, 0, 128]),
        ))
    }

    #[test]
    fn overlay_opaque_replaces_base() {
        let base = red_image(8, 8);
        let layer = blue_image(4, 4);
        let op = OverlayOp::new(layer, 0, 0, 1.0).unwrap();
        let result = op.apply(base).unwrap();
        let rgba = result.to_rgba8();

        // Top-left should be blue
        let p = rgba.get_pixel(2, 2);
        assert_eq!(p[0], 0);
        assert_eq!(p[2], 255);

        // Bottom-right should still be red
        let p2 = rgba.get_pixel(6, 6);
        assert_eq!(p2[0], 255);
        assert_eq!(p2[2], 0);
    }

    #[test]
    fn overlay_with_opacity() {
        let base = red_image(8, 8);
        let layer = blue_image(8, 8);
        let op = OverlayOp::new(layer, 0, 0, 0.5).unwrap();
        let result = op.apply(base).unwrap();
        let rgba = result.to_rgba8();

        let p = rgba.get_pixel(4, 4);
        // Should be a blend of red and blue
        assert!(p[0] > 50 && p[0] < 200); // some red
        assert!(p[2] > 50 && p[2] < 200); // some blue
    }

    #[test]
    fn overlay_with_offset() {
        let base = red_image(8, 8);
        let layer = blue_image(4, 4);
        let op = OverlayOp::new(layer, 4, 4, 1.0).unwrap();
        let result = op.apply(base).unwrap();
        let rgba = result.to_rgba8();

        // (2,2) should still be red
        let p = rgba.get_pixel(2, 2);
        assert_eq!(p[0], 255);
        assert_eq!(p[2], 0);

        // (5,5) should be blue
        let p2 = rgba.get_pixel(5, 5);
        assert_eq!(p2[0], 0);
        assert_eq!(p2[2], 255);
    }

    #[test]
    fn overlay_semi_transparent() {
        let base = red_image(8, 8);
        let layer = semi_transparent_green(8, 8);
        let op = OverlayOp::new(layer, 0, 0, 1.0).unwrap();
        let result = op.apply(base).unwrap();
        let rgba = result.to_rgba8();

        let p = rgba.get_pixel(4, 4);
        // Should have some red and some green
        assert!(p[0] > 50); // red component
        assert!(p[1] > 50); // green component
    }

    #[test]
    fn overlay_negative_offset() {
        let base = red_image(8, 8);
        let layer = blue_image(4, 4);
        // Layer starts at (-2, -2), only bottom-right 2x2 visible
        let op = OverlayOp::new(layer, -2, -2, 1.0).unwrap();
        let result = op.apply(base).unwrap();
        let rgba = result.to_rgba8();

        // (0,0) and (1,1) should be blue
        assert_eq!(rgba.get_pixel(0, 0)[2], 255);
        assert_eq!(rgba.get_pixel(1, 1)[2], 255);
        // (3,3) should be red
        assert_eq!(rgba.get_pixel(3, 3)[0], 255);
    }

    #[test]
    fn overlay_invalid_opacity() {
        let layer = blue_image(4, 4);
        assert!(OverlayOp::new(layer.clone(), 0, 0, -0.1).is_err());
        assert!(OverlayOp::new(layer, 0, 0, 1.1).is_err());
    }

    #[test]
    fn resolve_position_center() {
        let (x, y) = resolve_position("center", 100, 100, 20, 20, 0).unwrap();
        assert_eq!(x, 40);
        assert_eq!(y, 40);
    }

    #[test]
    fn resolve_position_corners() {
        let margin = 10;
        let (x, y) = resolve_position("top-left", 100, 100, 20, 20, margin).unwrap();
        assert_eq!(x, 10);
        assert_eq!(y, 10);

        let (x, y) = resolve_position("bottom-right", 100, 100, 20, 20, margin).unwrap();
        assert_eq!(x, 70);
        assert_eq!(y, 70);
    }

    #[test]
    fn resolve_position_unknown() {
        assert!(resolve_position("middle", 100, 100, 20, 20, 0).is_err());
    }
}
