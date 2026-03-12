use crate::error::{PanimgError, Result};
use crate::ops::{Operation, OperationDescription};
use crate::schema::{CommandSchema, ParamSchema, ParamType};
use image::{DynamicImage, Rgba, RgbaImage};

fn put_blended(img: &mut RgbaImage, x: u32, y: u32, color: &Rgba<u8>) {
    if x < img.width() && y < img.height() {
        let base = *img.get_pixel(x, y);
        img.put_pixel(x, y, super::blend_pixel(&base, color, 1.0));
    }
}

/// Draw a filled or outlined rectangle.
pub struct DrawRectOp {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub color: Rgba<u8>,
    pub fill: bool,
    pub thickness: u32,
}

impl DrawRectOp {
    pub fn new(
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        color: Rgba<u8>,
        fill: bool,
        thickness: u32,
    ) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(PanimgError::InvalidArgument {
                message: "rectangle width and height must be > 0".into(),
                suggestion: "e.g. --width 100 --height 50".into(),
            });
        }
        Ok(Self {
            x,
            y,
            width,
            height,
            color,
            fill,
            thickness,
        })
    }
}

impl Operation for DrawRectOp {
    fn name(&self) -> &str {
        "draw-rect"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        let mut rgba = img.to_rgba8();
        let (iw, ih) = (rgba.width(), rgba.height());

        if self.fill {
            for dy in 0..self.height {
                let py = self.y + dy as i32;
                if py < 0 || py >= ih as i32 {
                    continue;
                }
                for dx in 0..self.width {
                    let px = self.x + dx as i32;
                    if px < 0 || px >= iw as i32 {
                        continue;
                    }
                    put_blended(&mut rgba, px as u32, py as u32, &self.color);
                }
            }
        } else {
            let t = self.thickness;
            // Top and bottom edges
            for dy in 0..t {
                for dx in 0..self.width {
                    let px = self.x + dx as i32;
                    // Top
                    let py_top = self.y + dy as i32;
                    if px >= 0 && px < iw as i32 && py_top >= 0 && py_top < ih as i32 {
                        put_blended(&mut rgba, px as u32, py_top as u32, &self.color);
                    }
                    // Bottom
                    let py_bot = self.y + self.height as i32 - 1 - dy as i32;
                    if px >= 0 && px < iw as i32 && py_bot >= 0 && py_bot < ih as i32 {
                        put_blended(&mut rgba, px as u32, py_bot as u32, &self.color);
                    }
                }
            }
            // Left and right edges
            for dy in t..self.height.saturating_sub(t) {
                let py = self.y + dy as i32;
                if py < 0 || py >= ih as i32 {
                    continue;
                }
                for d in 0..t {
                    // Left
                    let px_left = self.x + d as i32;
                    if px_left >= 0 && px_left < iw as i32 {
                        put_blended(&mut rgba, px_left as u32, py as u32, &self.color);
                    }
                    // Right
                    let px_right = self.x + self.width as i32 - 1 - d as i32;
                    if px_right >= 0 && px_right < iw as i32 {
                        put_blended(&mut rgba, px_right as u32, py as u32, &self.color);
                    }
                }
            }
        }

        Ok(DynamicImage::ImageRgba8(rgba))
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "draw-rect".into(),
            params: serde_json::json!({
                "x": self.x,
                "y": self.y,
                "width": self.width,
                "height": self.height,
                "fill": self.fill,
            }),
            description: format!(
                "Draw {} rectangle at ({},{}) {}x{}",
                if self.fill { "filled" } else { "outlined" },
                self.x,
                self.y,
                self.width,
                self.height
            ),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "draw".into(),
            description: "Draw shapes on an image".into(),
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
                    name: "shape".into(),
                    param_type: ParamType::String,
                    required: true,
                    description: "Shape type: rect, circle, line".into(),
                    default: None,
                    choices: Some(vec!["rect".into(), "circle".into(), "line".into()]),
                    range: None,
                },
                ParamSchema {
                    name: "color".into(),
                    param_type: ParamType::String,
                    required: false,
                    description: "Color: hex (#FF0000), RGB (255,0,0), or named (red, blue, etc.)"
                        .into(),
                    default: Some(serde_json::json!("red")),
                    choices: None,
                    range: None,
                },
            ],
        }
    }
}

/// Draw a filled or outlined circle.
pub struct DrawCircleOp {
    pub cx: i32,
    pub cy: i32,
    pub radius: u32,
    pub color: Rgba<u8>,
    pub fill: bool,
    pub thickness: u32,
}

impl DrawCircleOp {
    pub fn new(
        cx: i32,
        cy: i32,
        radius: u32,
        color: Rgba<u8>,
        fill: bool,
        thickness: u32,
    ) -> Result<Self> {
        if radius == 0 {
            return Err(PanimgError::InvalidArgument {
                message: "circle radius must be > 0".into(),
                suggestion: "e.g. --radius 30".into(),
            });
        }
        Ok(Self {
            cx,
            cy,
            radius,
            color,
            fill,
            thickness,
        })
    }
}

impl Operation for DrawCircleOp {
    fn name(&self) -> &str {
        "draw-circle"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        let mut rgba = img.to_rgba8();
        let r = self.radius as i32;

        if self.fill {
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx * dx + dy * dy <= r * r {
                        let px = self.cx + dx;
                        let py = self.cy + dy;
                        if px >= 0 && py >= 0 {
                            put_blended(&mut rgba, px as u32, py as u32, &self.color);
                        }
                    }
                }
            }
        } else {
            let outer_r = r;
            let inner_r = (r - self.thickness as i32).max(0);
            let outer_r2 = outer_r * outer_r;
            let inner_r2 = inner_r * inner_r;
            for dy in -outer_r..=outer_r {
                for dx in -outer_r..=outer_r {
                    let d2 = dx * dx + dy * dy;
                    if d2 <= outer_r2 && d2 >= inner_r2 {
                        let px = self.cx + dx;
                        let py = self.cy + dy;
                        if px >= 0 && py >= 0 {
                            put_blended(&mut rgba, px as u32, py as u32, &self.color);
                        }
                    }
                }
            }
        }

        Ok(DynamicImage::ImageRgba8(rgba))
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "draw-circle".into(),
            params: serde_json::json!({
                "cx": self.cx,
                "cy": self.cy,
                "radius": self.radius,
                "fill": self.fill,
            }),
            description: format!(
                "Draw {} circle at ({},{}) r={}",
                if self.fill { "filled" } else { "outlined" },
                self.cx,
                self.cy,
                self.radius
            ),
        }
    }

    fn schema() -> CommandSchema {
        DrawRectOp::schema() // Same top-level schema
    }
}

/// Draw a line using Bresenham's algorithm with thickness.
pub struct DrawLineOp {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
    pub color: Rgba<u8>,
    pub thickness: u32,
}

impl DrawLineOp {
    pub fn new(x1: i32, y1: i32, x2: i32, y2: i32, color: Rgba<u8>, thickness: u32) -> Self {
        Self {
            x1,
            y1,
            x2,
            y2,
            color,
            thickness,
        }
    }
}

impl Operation for DrawLineOp {
    fn name(&self) -> &str {
        "draw-line"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        let mut rgba = img.to_rgba8();
        let half_t = self.thickness as i32 / 2;

        // Bresenham's line algorithm
        let mut x0 = self.x1;
        let mut y0 = self.y1;
        let x1 = self.x2;
        let y1 = self.y2;

        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            // Draw a filled circle at each point for thickness
            for ty in -half_t..=half_t {
                for tx in -half_t..=half_t {
                    if tx * tx + ty * ty <= half_t * half_t + half_t {
                        let px = x0 + tx;
                        let py = y0 + ty;
                        if px >= 0 && py >= 0 {
                            put_blended(&mut rgba, px as u32, py as u32, &self.color);
                        }
                    }
                }
            }

            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }

        Ok(DynamicImage::ImageRgba8(rgba))
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "draw-line".into(),
            params: serde_json::json!({
                "x1": self.x1,
                "y1": self.y1,
                "x2": self.x2,
                "y2": self.y2,
                "thickness": self.thickness,
            }),
            description: format!(
                "Draw line from ({},{}) to ({},{}) thickness={}",
                self.x1, self.y1, self.x2, self.y2, self.thickness
            ),
        }
    }

    fn schema() -> CommandSchema {
        DrawRectOp::schema() // Same top-level schema
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn white_image(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            w,
            h,
            image::Rgba([255, 255, 255, 255]),
        ))
    }

    #[test]
    fn draw_filled_rect() {
        let img = white_image(10, 10);
        let op = DrawRectOp::new(2, 2, 4, 4, Rgba([255, 0, 0, 255]), true, 1).unwrap();
        let result = op.apply(img).unwrap();
        let rgba = result.to_rgba8();
        // Inside rect should be red
        assert_eq!(rgba.get_pixel(3, 3)[0], 255);
        assert_eq!(rgba.get_pixel(3, 3)[2], 0);
        // Outside should be white
        assert_eq!(rgba.get_pixel(0, 0)[0], 255);
        assert_eq!(rgba.get_pixel(0, 0)[1], 255);
    }

    #[test]
    fn draw_outlined_rect() {
        let img = white_image(20, 20);
        let op = DrawRectOp::new(5, 5, 10, 10, Rgba([255, 0, 0, 255]), false, 1).unwrap();
        let result = op.apply(img).unwrap();
        let rgba = result.to_rgba8();
        // Border pixel should be red
        assert_eq!(rgba.get_pixel(5, 5)[0], 255);
        assert_eq!(rgba.get_pixel(5, 5)[1], 0);
        // Interior should be white
        assert_eq!(rgba.get_pixel(10, 10)[0], 255);
        assert_eq!(rgba.get_pixel(10, 10)[1], 255);
    }

    #[test]
    fn draw_filled_circle() {
        let img = white_image(20, 20);
        let op = DrawCircleOp::new(10, 10, 5, Rgba([0, 0, 255, 255]), true, 1).unwrap();
        let result = op.apply(img).unwrap();
        let rgba = result.to_rgba8();
        // Center should be blue
        assert_eq!(rgba.get_pixel(10, 10)[2], 255);
        assert_eq!(rgba.get_pixel(10, 10)[0], 0);
        // Corner should be white
        assert_eq!(rgba.get_pixel(0, 0)[0], 255);
    }

    #[test]
    fn draw_line() {
        let img = white_image(10, 10);
        let op = DrawLineOp::new(0, 0, 9, 9, Rgba([255, 0, 0, 255]), 1);
        let result = op.apply(img).unwrap();
        let rgba = result.to_rgba8();
        // Diagonal pixels should be red
        assert_eq!(rgba.get_pixel(5, 5)[0], 255);
        assert_eq!(rgba.get_pixel(5, 5)[1], 0);
    }

    #[test]
    fn draw_with_alpha() {
        let img = white_image(10, 10);
        let op = DrawRectOp::new(0, 0, 10, 10, Rgba([255, 0, 0, 128]), true, 1).unwrap();
        let result = op.apply(img).unwrap();
        let rgba = result.to_rgba8();
        // Should be a blend of red and white
        let p = rgba.get_pixel(5, 5);
        assert!(p[0] > 200); // Still high red
        assert!(p[1] > 50 && p[1] < 200); // Some green from white
    }

    #[test]
    fn draw_preserves_dimensions() {
        let img = white_image(10, 10);
        let op = DrawRectOp::new(2, 2, 4, 4, Rgba([255, 0, 0, 255]), true, 1).unwrap();
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 10);
        assert_eq!(result.height(), 10);
    }

    #[test]
    fn draw_rect_zero_size() {
        assert!(DrawRectOp::new(0, 0, 0, 10, Rgba([255, 0, 0, 255]), true, 1).is_err());
    }

    #[test]
    fn draw_circle_zero_radius() {
        assert!(DrawCircleOp::new(0, 0, 0, Rgba([255, 0, 0, 255]), true, 1).is_err());
    }
}
