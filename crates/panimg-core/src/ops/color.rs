use crate::error::Result;
use crate::ops::{Operation, OperationDescription};
use crate::schema::{CommandSchema, ParamSchema, ParamType};
use image::DynamicImage;

/// Adjust color saturation of an image.
/// factor > 1.0 increases saturation, < 1.0 decreases, 0.0 = grayscale.
pub struct SaturateOp {
    factor: f32,
}

impl SaturateOp {
    pub fn new(factor: f32) -> Result<Self> {
        Ok(Self { factor })
    }
}

impl Operation for SaturateOp {
    fn name(&self) -> &str {
        "saturate"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        let rgba = img.to_rgba8();
        let mut out = rgba.clone();

        for (x, y, pixel) in rgba.enumerate_pixels() {
            let [r, g, b, a] = pixel.0;
            let (h, s, l) = rgb_to_hsl(r, g, b);
            let new_s = (s * self.factor).clamp(0.0, 1.0);
            let (nr, ng, nb) = hsl_to_rgb(h, new_s, l);
            out.put_pixel(x, y, image::Rgba([nr, ng, nb, a]));
        }

        Ok(DynamicImage::ImageRgba8(out))
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "saturate".into(),
            params: serde_json::json!({ "factor": self.factor }),
            description: format!("Adjust saturation by factor {}", self.factor),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "saturate".into(),
            description: "Adjust color saturation".into(),
            params: vec![ParamSchema {
                name: "factor".into(),
                param_type: ParamType::Float,
                required: true,
                description: "Saturation factor (0.0=grayscale, 1.0=unchanged, 2.0=double)".into(),
                default: None,
                choices: None,
                range: None,
            }],
        }
    }
}

/// Apply sepia tone effect.
pub struct SepiaOp {
    intensity: f32,
}

impl SepiaOp {
    pub fn new(intensity: f32) -> Result<Self> {
        Ok(Self {
            intensity: intensity.clamp(0.0, 1.0),
        })
    }
}

impl Operation for SepiaOp {
    fn name(&self) -> &str {
        "sepia"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        let rgba = img.to_rgba8();
        let mut out = rgba.clone();
        let t = self.intensity;

        for (x, y, pixel) in rgba.enumerate_pixels() {
            let [r, g, b, a] = pixel.0;
            let rf = r as f32;
            let gf = g as f32;
            let bf = b as f32;

            // Standard sepia matrix
            let sr = (0.393 * rf + 0.769 * gf + 0.189 * bf).min(255.0);
            let sg = (0.349 * rf + 0.686 * gf + 0.168 * bf).min(255.0);
            let sb = (0.272 * rf + 0.534 * gf + 0.131 * bf).min(255.0);

            // Blend between original and sepia
            let nr = (rf * (1.0 - t) + sr * t) as u8;
            let ng = (gf * (1.0 - t) + sg * t) as u8;
            let nb = (bf * (1.0 - t) + sb * t) as u8;

            out.put_pixel(x, y, image::Rgba([nr, ng, nb, a]));
        }

        Ok(DynamicImage::ImageRgba8(out))
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "sepia".into(),
            params: serde_json::json!({ "intensity": self.intensity }),
            description: format!("Apply sepia tone at intensity {}", self.intensity),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "sepia".into(),
            description: "Apply sepia tone effect".into(),
            params: vec![ParamSchema {
                name: "intensity".into(),
                param_type: ParamType::Float,
                required: false,
                description: "Sepia intensity (0.0-1.0, default: 1.0)".into(),
                default: Some(serde_json::json!(1.0)),
                choices: None,
                range: None,
            }],
        }
    }
}

/// Tint an image with a specific color.
pub struct TintOp {
    r: u8,
    g: u8,
    b: u8,
    strength: f32,
}

impl TintOp {
    pub fn new(r: u8, g: u8, b: u8, strength: f32) -> Result<Self> {
        Ok(Self {
            r,
            g,
            b,
            strength: strength.clamp(0.0, 1.0),
        })
    }
}

impl Operation for TintOp {
    fn name(&self) -> &str {
        "tint"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        let rgba = img.to_rgba8();
        let mut out = rgba.clone();
        let t = self.strength;

        for (x, y, pixel) in rgba.enumerate_pixels() {
            let [r, g, b, a] = pixel.0;
            // Multiply blend: tint color modulates original
            let nr = ((r as f32 * (1.0 - t)) + (r as f32 * self.r as f32 / 255.0 * t)) as u8;
            let ng = ((g as f32 * (1.0 - t)) + (g as f32 * self.g as f32 / 255.0 * t)) as u8;
            let nb = ((b as f32 * (1.0 - t)) + (b as f32 * self.b as f32 / 255.0 * t)) as u8;
            out.put_pixel(x, y, image::Rgba([nr, ng, nb, a]));
        }

        Ok(DynamicImage::ImageRgba8(out))
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "tint".into(),
            params: serde_json::json!({
                "color": format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b),
                "strength": self.strength
            }),
            description: format!(
                "Tint with #{:02X}{:02X}{:02X} at strength {}",
                self.r, self.g, self.b, self.strength
            ),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "tint".into(),
            description: "Tint image with a color".into(),
            params: vec![
                ParamSchema {
                    name: "color".into(),
                    param_type: ParamType::String,
                    required: true,
                    description: "Tint color (hex, RGB, or named)".into(),
                    default: None,
                    choices: None,
                    range: None,
                },
                ParamSchema {
                    name: "strength".into(),
                    param_type: ParamType::Float,
                    required: false,
                    description: "Tint strength (0.0-1.0, default: 0.5)".into(),
                    default: Some(serde_json::json!(0.5)),
                    choices: None,
                    range: None,
                },
            ],
        }
    }
}

/// Posterize an image by reducing color levels per channel.
pub struct PosterizeOp {
    levels: u8,
}

impl PosterizeOp {
    pub fn new(levels: u8) -> Result<Self> {
        Ok(Self {
            levels: levels.max(2),
        })
    }
}

impl Operation for PosterizeOp {
    fn name(&self) -> &str {
        "posterize"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        let rgba = img.to_rgba8();
        let mut out = rgba.clone();
        let levels = self.levels as f32;
        let step = 255.0 / (levels - 1.0);

        for (x, y, pixel) in rgba.enumerate_pixels() {
            let [r, g, b, a] = pixel.0;
            let nr = ((r as f32 / 255.0 * (levels - 1.0)).round() * step) as u8;
            let ng = ((g as f32 / 255.0 * (levels - 1.0)).round() * step) as u8;
            let nb = ((b as f32 / 255.0 * (levels - 1.0)).round() * step) as u8;
            out.put_pixel(x, y, image::Rgba([nr, ng, nb, a]));
        }

        Ok(DynamicImage::ImageRgba8(out))
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "posterize".into(),
            params: serde_json::json!({ "levels": self.levels }),
            description: format!("Posterize to {} levels per channel", self.levels),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "posterize".into(),
            description: "Reduce color levels per channel".into(),
            params: vec![ParamSchema {
                name: "levels".into(),
                param_type: ParamType::Integer,
                required: false,
                description: "Number of color levels per channel (2-256, default: 4)".into(),
                default: Some(serde_json::json!(4)),
                choices: None,
                range: None,
            }],
        }
    }
}

// --- HSL helper functions ---

fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;

    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let l = (max + min) / 2.0;

    if (max - min).abs() < f32::EPSILON {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if (max - rf).abs() < f32::EPSILON {
        let mut h = (gf - bf) / d;
        if gf < bf {
            h += 6.0;
        }
        h
    } else if (max - gf).abs() < f32::EPSILON {
        (bf - rf) / d + 2.0
    } else {
        (rf - gf) / d + 4.0
    };

    (h / 6.0, s, l)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    if s.abs() < f32::EPSILON {
        let v = (l * 255.0) as u8;
        return (v, v, v);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, RgbaImage};

    fn test_image() -> DynamicImage {
        DynamicImage::ImageRgba8(RgbaImage::from_pixel(
            4,
            4,
            image::Rgba([200, 100, 50, 255]),
        ))
    }

    #[test]
    fn saturate_preserves_dimensions() {
        let img = test_image();
        let op = SaturateOp::new(1.5).unwrap();
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 4);
        assert_eq!(result.height(), 4);
    }

    #[test]
    fn saturate_zero_produces_gray() {
        let img = test_image();
        let op = SaturateOp::new(0.0).unwrap();
        let result = op.apply(img).unwrap();
        let rgba = result.to_rgba8();
        let p = rgba.get_pixel(0, 0);
        // With zero saturation, R == G == B (grayscale)
        assert_eq!(p[0], p[1]);
        assert_eq!(p[1], p[2]);
    }

    #[test]
    fn saturate_one_is_identity() {
        let img = test_image();
        let op = SaturateOp::new(1.0).unwrap();
        let result = op.apply(img).unwrap();
        let rgba = result.to_rgba8();
        let p = rgba.get_pixel(0, 0);
        // Should be very close to original (200, 100, 50)
        assert!((p[0] as i32 - 200).unsigned_abs() <= 1);
        assert!((p[1] as i32 - 100).unsigned_abs() <= 1);
        assert!((p[2] as i32 - 50).unsigned_abs() <= 1);
    }

    #[test]
    fn sepia_changes_colors() {
        let img = test_image();
        let op = SepiaOp::new(1.0).unwrap();
        let result = op.apply(img).unwrap();
        let rgba = result.to_rgba8();
        let p = rgba.get_pixel(0, 0);
        // Sepia should produce a warm tone (R > G > B)
        assert!(p[0] >= p[1]);
        assert!(p[1] >= p[2]);
    }

    #[test]
    fn sepia_zero_is_identity() {
        let img = test_image();
        let op = SepiaOp::new(0.0).unwrap();
        let result = op.apply(img).unwrap();
        let rgba = result.to_rgba8();
        let p = rgba.get_pixel(0, 0);
        assert_eq!(p[0], 200);
        assert_eq!(p[1], 100);
        assert_eq!(p[2], 50);
    }

    #[test]
    fn tint_preserves_dimensions() {
        let img = test_image();
        let op = TintOp::new(255, 0, 0, 0.5).unwrap();
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 4);
        assert_eq!(result.height(), 4);
    }

    #[test]
    fn posterize_reduces_colors() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_fn(4, 4, |x, _| {
            let v = (x * 85) as u8; // 0, 85, 170, 255
            image::Rgba([v, v, v, 255])
        }));
        let op = PosterizeOp::new(2).unwrap();
        let result = op.apply(img).unwrap();
        let rgba = result.to_rgba8();
        // With 2 levels, every pixel should be either 0 or 255
        for p in rgba.pixels() {
            assert!(p[0] == 0 || p[0] == 255);
        }
    }

    #[test]
    fn posterize_preserves_dimensions() {
        let img = test_image();
        let op = PosterizeOp::new(4).unwrap();
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 4);
        assert_eq!(result.height(), 4);
    }

    #[test]
    fn hsl_roundtrip() {
        let (h, s, l) = rgb_to_hsl(200, 100, 50);
        let (r, g, b) = hsl_to_rgb(h, s, l);
        assert!((r as i32 - 200).unsigned_abs() <= 1);
        assert!((g as i32 - 100).unsigned_abs() <= 1);
        assert!((b as i32 - 50).unsigned_abs() <= 1);
    }

    #[test]
    fn hsl_gray() {
        let (_, s, _) = rgb_to_hsl(128, 128, 128);
        assert!(s < 0.01);
    }
}
