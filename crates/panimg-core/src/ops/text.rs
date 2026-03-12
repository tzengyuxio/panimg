use crate::error::{PanimgError, Result};
use crate::ops::overlay::resolve_position;
use crate::ops::{Operation, OperationDescription};
use crate::schema::{CommandSchema, ParamRange, ParamSchema, ParamType};
use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use image::{DynamicImage, Rgba, RgbaImage};

/// Default embedded font (DejaVu Sans).
const DEFAULT_FONT_BYTES: &[u8] = include_bytes!("../fonts/DejaVuSans.ttf");

/// Draw text on an image.
pub struct DrawTextOp {
    pub content: String,
    pub font_bytes: Vec<u8>,
    pub size: f32,
    pub color: Rgba<u8>,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub position: Option<String>,
    pub margin: u32,
}

impl DrawTextOp {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        content: String,
        font_path: Option<&str>,
        size: f32,
        color: Rgba<u8>,
        x: Option<i32>,
        y: Option<i32>,
        position: Option<String>,
        margin: u32,
    ) -> Result<Self> {
        if content.is_empty() {
            return Err(PanimgError::InvalidArgument {
                message: "text content cannot be empty".into(),
                suggestion: "provide text with --content \"Hello World\"".into(),
            });
        }
        if size <= 0.0 {
            return Err(PanimgError::InvalidArgument {
                message: format!("font size must be positive, got {size}"),
                suggestion: "use a value like --size 24".into(),
            });
        }

        let font_bytes = match font_path {
            Some(path) => std::fs::read(path).map_err(|e| PanimgError::IoError {
                path: Some(path.into()),
                message: format!("failed to read font file: {e}"),
                suggestion: "check that the font file exists and is readable".into(),
            })?,
            None => DEFAULT_FONT_BYTES.to_vec(),
        };

        // Validate font can be parsed
        FontRef::try_from_slice(&font_bytes).map_err(|e| PanimgError::InvalidArgument {
            message: format!("invalid font file: {e}"),
            suggestion: "provide a valid TTF or OTF font file".into(),
        })?;

        Ok(Self {
            content,
            font_bytes,
            size,
            color,
            x,
            y,
            position,
            margin,
        })
    }
}

/// Measure the bounding box of rendered text.
fn measure_text(font: &FontRef, scale: PxScale, text: &str) -> (u32, u32) {
    let scaled = font.as_scaled(scale);
    let mut width: f32 = 0.0;
    let mut prev_glyph_id = None;

    for ch in text.chars() {
        let glyph_id = font.glyph_id(ch);
        if let Some(prev) = prev_glyph_id {
            width += scaled.kern(prev, glyph_id);
        }
        width += scaled.h_advance(glyph_id);
        prev_glyph_id = Some(glyph_id);
    }

    let height = scaled.ascent() - scaled.descent();
    (width.ceil() as u32, height.ceil() as u32)
}

/// Blend a color onto a pixel with alpha compositing.
fn blend_pixel(base: &Rgba<u8>, color: &Rgba<u8>, coverage: f32) -> Rgba<u8> {
    let ca = (color[3] as f32 / 255.0) * coverage;
    let ba = base[3] as f32 / 255.0;
    let out_a = ca + ba * (1.0 - ca);
    if out_a == 0.0 {
        return Rgba([0, 0, 0, 0]);
    }
    let blend = |cc: u8, bc: u8| -> u8 {
        let c = (cc as f32 / 255.0 * ca + bc as f32 / 255.0 * ba * (1.0 - ca)) / out_a;
        (c * 255.0).round().clamp(0.0, 255.0) as u8
    };
    Rgba([
        blend(color[0], base[0]),
        blend(color[1], base[1]),
        blend(color[2], base[2]),
        (out_a * 255.0).round().clamp(0.0, 255.0) as u8,
    ])
}

impl Operation for DrawTextOp {
    fn name(&self) -> &str {
        "text"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        let mut rgba = img.to_rgba8();
        let (img_w, img_h) = (rgba.width(), rgba.height());

        let font = FontRef::try_from_slice(&self.font_bytes).map_err(|e| {
            PanimgError::InvalidArgument {
                message: format!("failed to parse font: {e}"),
                suggestion: "provide a valid TTF or OTF font file".into(),
            }
        })?;

        let scale = PxScale::from(self.size);
        let (text_w, text_h) = measure_text(&font, scale, &self.content);

        // Determine position
        let (draw_x, draw_y) = if let (Some(x), Some(y)) = (self.x, self.y) {
            (x as i64, y as i64)
        } else if let Some(ref pos) = self.position {
            resolve_position(pos, img_w, img_h, text_w, text_h, self.margin as i64)?
        } else if let Some(x) = self.x {
            (x as i64, self.margin as i64)
        } else if let Some(y) = self.y {
            (self.margin as i64, y as i64)
        } else {
            (self.margin as i64, self.margin as i64)
        };

        // Render glyphs
        draw_text_on_image(
            &mut rgba,
            &font,
            scale,
            &self.content,
            draw_x,
            draw_y,
            &self.color,
        );

        Ok(DynamicImage::ImageRgba8(rgba))
    }

    fn describe(&self) -> OperationDescription {
        let mut params = serde_json::json!({
            "content": self.content,
            "size": self.size,
        });
        if let Some(ref pos) = self.position {
            params["position"] = serde_json::json!(pos);
            params["margin"] = serde_json::json!(self.margin);
        }
        if let Some(x) = self.x {
            params["x"] = serde_json::json!(x);
        }
        if let Some(y) = self.y {
            params["y"] = serde_json::json!(y);
        }

        OperationDescription {
            operation: "text".into(),
            params,
            description: format!(
                "Draw text \"{}\" at {} size={}",
                self.content,
                if let Some(ref pos) = self.position {
                    format!("position={pos}")
                } else {
                    format!(
                        "({},{})",
                        self.x.unwrap_or(self.margin as i32),
                        self.y.unwrap_or(self.margin as i32)
                    )
                },
                self.size
            ),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "text".into(),
            description: "Draw text on an image".into(),
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
                    name: "content".into(),
                    param_type: ParamType::String,
                    required: true,
                    description: "Text content to draw".into(),
                    default: None,
                    choices: None,
                    range: None,
                },
                ParamSchema {
                    name: "font".into(),
                    param_type: ParamType::Path,
                    required: false,
                    description: "TTF/OTF font file path (uses embedded DejaVu Sans if omitted)"
                        .into(),
                    default: None,
                    choices: None,
                    range: None,
                },
                ParamSchema {
                    name: "size".into(),
                    param_type: ParamType::Float,
                    required: false,
                    description: "Font size in pixels".into(),
                    default: Some(serde_json::json!(24.0)),
                    choices: None,
                    range: Some(ParamRange {
                        min: 1.0,
                        max: 1000.0,
                    }),
                },
                ParamSchema {
                    name: "color".into(),
                    param_type: ParamType::String,
                    required: false,
                    description:
                        "Text color: hex (#FFFFFF), RGB (255,255,255), or named (white, red, etc.)"
                            .into(),
                    default: Some(serde_json::json!("white")),
                    choices: None,
                    range: None,
                },
                ParamSchema {
                    name: "x".into(),
                    param_type: ParamType::Integer,
                    required: false,
                    description: "Absolute X position (overrides --position)".into(),
                    default: None,
                    choices: None,
                    range: None,
                },
                ParamSchema {
                    name: "y".into(),
                    param_type: ParamType::Integer,
                    required: false,
                    description: "Absolute Y position (overrides --position)".into(),
                    default: None,
                    choices: None,
                    range: None,
                },
                ParamSchema {
                    name: "position".into(),
                    param_type: ParamType::String,
                    required: false,
                    description:
                        "Named position: center, top-left, top-right, bottom-left, bottom-right"
                            .into(),
                    default: Some(serde_json::json!("top-left")),
                    choices: Some(vec![
                        "center".into(),
                        "top-left".into(),
                        "top-right".into(),
                        "bottom-left".into(),
                        "bottom-right".into(),
                    ]),
                    range: None,
                },
                ParamSchema {
                    name: "margin".into(),
                    param_type: ParamType::Integer,
                    required: false,
                    description: "Margin in pixels for named positions".into(),
                    default: Some(serde_json::json!(10)),
                    choices: None,
                    range: Some(ParamRange {
                        min: 0.0,
                        max: 10000.0,
                    }),
                },
            ],
        }
    }
}

/// Render text onto an RGBA image at a given position.
fn draw_text_on_image(
    img: &mut RgbaImage,
    font: &FontRef,
    scale: PxScale,
    text: &str,
    start_x: i64,
    start_y: i64,
    color: &Rgba<u8>,
) {
    use ab_glyph::Font as _;

    let scaled = font.as_scaled(scale);
    let ascent = scaled.ascent();

    let mut cursor_x: f32 = 0.0;
    let mut prev_glyph_id = None;

    let (img_w, img_h) = (img.width() as i64, img.height() as i64);

    for ch in text.chars() {
        let glyph_id = font.glyph_id(ch);

        if let Some(prev) = prev_glyph_id {
            cursor_x += scaled.kern(prev, glyph_id);
        }

        let glyph = glyph_id.with_scale_and_position(scale, ab_glyph::point(cursor_x, ascent));

        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|gx, gy, coverage| {
                let px = start_x + bounds.min.x as i64 + gx as i64;
                let py = start_y + bounds.min.y as i64 + gy as i64;

                if px >= 0 && px < img_w && py >= 0 && py < img_h {
                    let base = *img.get_pixel(px as u32, py as u32);
                    let blended = blend_pixel(&base, color, coverage);
                    img.put_pixel(px as u32, py as u32, blended);
                }
            });
        }

        cursor_x += scaled.h_advance(glyph_id);
        prev_glyph_id = Some(glyph_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn white_image(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(RgbaImage::from_pixel(w, h, Rgba([255, 255, 255, 255])))
    }

    #[test]
    fn draw_text_basic() {
        let img = white_image(200, 100);
        let op = DrawTextOp::new(
            "Hello".into(),
            None,
            24.0,
            Rgba([0, 0, 0, 255]),
            Some(10),
            Some(10),
            None,
            10,
        )
        .unwrap();
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 200);
        assert_eq!(result.height(), 100);

        // Some pixels should have been modified (text drawn)
        let rgba = result.to_rgba8();
        let mut has_dark_pixel = false;
        for y in 10..40 {
            for x in 10..100 {
                let p = rgba.get_pixel(x, y);
                if p[0] < 200 {
                    has_dark_pixel = true;
                    break;
                }
            }
        }
        assert!(has_dark_pixel, "text should have been rendered");
    }

    #[test]
    fn draw_text_with_position() {
        let img = white_image(200, 100);
        let op = DrawTextOp::new(
            "Test".into(),
            None,
            20.0,
            Rgba([255, 0, 0, 255]),
            None,
            None,
            Some("center".into()),
            10,
        )
        .unwrap();
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 200);
        assert_eq!(result.height(), 100);
    }

    #[test]
    fn draw_text_preserves_dimensions() {
        let img = white_image(100, 50);
        let op = DrawTextOp::new(
            "X".into(),
            None,
            12.0,
            Rgba([255, 255, 255, 255]),
            Some(0),
            Some(0),
            None,
            0,
        )
        .unwrap();
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 100);
        assert_eq!(result.height(), 50);
    }

    #[test]
    fn draw_text_empty_content_error() {
        assert!(DrawTextOp::new(
            "".into(),
            None,
            24.0,
            Rgba([0, 0, 0, 255]),
            None,
            None,
            None,
            10,
        )
        .is_err());
    }

    #[test]
    fn draw_text_zero_size_error() {
        assert!(DrawTextOp::new(
            "Hello".into(),
            None,
            0.0,
            Rgba([0, 0, 0, 255]),
            None,
            None,
            None,
            10,
        )
        .is_err());
    }

    #[test]
    fn draw_text_invalid_font_error() {
        assert!(DrawTextOp::new(
            "Hello".into(),
            Some("/nonexistent/font.ttf"),
            24.0,
            Rgba([0, 0, 0, 255]),
            None,
            None,
            None,
            10,
        )
        .is_err());
    }

    #[test]
    fn draw_text_with_alpha() {
        let img = white_image(200, 100);
        let op = DrawTextOp::new(
            "Alpha".into(),
            None,
            24.0,
            Rgba([0, 0, 0, 128]),
            Some(10),
            Some(10),
            None,
            10,
        )
        .unwrap();
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 200);
    }

    #[test]
    fn draw_text_large_outside_bounds() {
        let img = white_image(50, 50);
        let op = DrawTextOp::new(
            "Very long text that overflows".into(),
            None,
            24.0,
            Rgba([0, 0, 0, 255]),
            Some(0),
            Some(0),
            None,
            0,
        )
        .unwrap();
        // Should not panic, just clip
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 50);
    }

    #[test]
    fn measure_text_gives_reasonable_size() {
        let font = FontRef::try_from_slice(DEFAULT_FONT_BYTES).unwrap();
        let scale = PxScale::from(24.0);
        let (w, h) = measure_text(&font, scale, "Hello");
        assert!(w > 0, "width should be positive");
        assert!(h > 0, "height should be positive");
        assert!(w < 200, "width should be reasonable");
        assert!(h < 50, "height should be reasonable");
    }

    #[test]
    fn describe_with_position() {
        let op = DrawTextOp::new(
            "Test".into(),
            None,
            24.0,
            Rgba([255, 255, 255, 255]),
            None,
            None,
            Some("bottom-right".into()),
            20,
        )
        .unwrap();
        let desc = op.describe();
        assert_eq!(desc.operation, "text");
        assert!(desc.description.contains("bottom-right"));
    }

    #[test]
    fn describe_with_coordinates() {
        let op = DrawTextOp::new(
            "Test".into(),
            None,
            48.0,
            Rgba([255, 0, 0, 255]),
            Some(100),
            Some(200),
            None,
            10,
        )
        .unwrap();
        let desc = op.describe();
        assert!(desc.description.contains("(100,200)"));
    }
}
