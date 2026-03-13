use crate::error::{PanimgError, Result};
use crate::ops::{Operation, OperationDescription};
use crate::schema::{CommandSchema, ParamSchema, ParamType};
use image::{DynamicImage, Rgba, RgbaImage};

/// Rotation angle — supports exact 90/180/270 (lossless fast path)
/// and arbitrary float angles.
#[derive(Debug, Clone, Copy)]
pub enum RotateAngle {
    Deg90,
    Deg180,
    Deg270,
    /// Arbitrary angle in degrees (clockwise).
    Arbitrary(f64),
}

impl RotateAngle {
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim() {
            "90" | "90cw" | "right" => Ok(Self::Deg90),
            "180" => Ok(Self::Deg180),
            "270" | "90ccw" | "left" => Ok(Self::Deg270),
            other => {
                // Try parsing as a float for arbitrary angles
                let deg: f64 =
                    other.parse().map_err(|_| PanimgError::InvalidArgument {
                        message: format!("unsupported rotation angle: '{other}'"),
                        suggestion:
                            "use 90, 180, 270, left, right, or any numeric angle (e.g. 45, 30.5)"
                                .into(),
                    })?;
                // Normalize and check if it matches a fast-path angle.
                // Use 1e-9 tolerance (not f64::EPSILON) because modulo arithmetic
                // on floats can introduce rounding error larger than machine epsilon.
                const ANGLE_TOL: f64 = 1e-9;
                let normalized = ((deg % 360.0) + 360.0) % 360.0;
                if (normalized - 90.0).abs() < ANGLE_TOL {
                    Ok(Self::Deg90)
                } else if (normalized - 180.0).abs() < ANGLE_TOL {
                    Ok(Self::Deg180)
                } else if (normalized - 270.0).abs() < ANGLE_TOL {
                    Ok(Self::Deg270)
                } else if normalized.abs() < ANGLE_TOL
                    || (normalized - 360.0).abs() < ANGLE_TOL
                {
                    // 0 or 360 degrees — effectively no rotation, but we still
                    // represent it as Arbitrary(0.0) so the caller gets a valid result.
                    Ok(Self::Arbitrary(0.0))
                } else {
                    Ok(Self::Arbitrary(deg))
                }
            }
        }
    }

    /// Return the angle in degrees for display purposes.
    pub fn degrees_f64(self) -> f64 {
        match self {
            Self::Deg90 => 90.0,
            Self::Deg180 => 180.0,
            Self::Deg270 => 270.0,
            Self::Arbitrary(d) => d,
        }
    }

    /// Return true if this is a cardinal (90/180/270) rotation.
    pub fn is_cardinal(self) -> bool {
        matches!(self, Self::Deg90 | Self::Deg180 | Self::Deg270)
    }
}

/// Rotate operation — supports 90/180/270 (fast lossless) and arbitrary angles.
pub struct RotateOp {
    pub angle: RotateAngle,
    /// Background fill color for areas exposed by arbitrary rotation.
    /// Ignored for cardinal rotations.
    pub background: Rgba<u8>,
}

impl RotateOp {
    pub fn new(angle: RotateAngle) -> Self {
        Self {
            angle,
            background: Rgba([0, 0, 0, 0]), // transparent by default
        }
    }

    pub fn with_background(mut self, bg: Rgba<u8>) -> Self {
        self.background = bg;
        self
    }
}

/// Rotate an RGBA image by an arbitrary angle (degrees, clockwise) using
/// bilinear interpolation, filling exposed areas with `background`.
fn rotate_arbitrary(img: &DynamicImage, degrees: f64, background: Rgba<u8>) -> DynamicImage {
    let src = img.to_rgba8();
    let (src_w, src_h) = (src.width() as f64, src.height() as f64);

    let rad = degrees.to_radians();
    let cos = rad.cos();
    let sin = rad.sin();

    // Compute bounding box of the rotated image
    let corners = [(0.0, 0.0), (src_w, 0.0), (0.0, src_h), (src_w, src_h)];
    let cx = src_w / 2.0;
    let cy = src_h / 2.0;

    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;
    for (px, py) in &corners {
        let dx = px - cx;
        let dy = py - cy;
        let rx = dx * cos - dy * sin;
        let ry = dx * sin + dy * cos;
        min_x = min_x.min(rx);
        max_x = max_x.max(rx);
        min_y = min_y.min(ry);
        max_y = max_y.max(ry);
    }

    let dst_w = (max_x - min_x).ceil() as u32;
    let dst_h = (max_y - min_y).ceil() as u32;
    let dst_cx = dst_w as f64 / 2.0;
    let dst_cy = dst_h as f64 / 2.0;

    let mut dst = RgbaImage::from_pixel(dst_w, dst_h, background);

    // Inverse rotation reuses already-computed trig values:
    // cos(-θ) == cos(θ), sin(-θ) == -sin(θ)
    let cos_inv = cos;
    let sin_inv = -sin;

    let src_w_u = src.width();
    let src_h_u = src.height();
    let src_w_f = src_w_u as f64;
    let src_h_f = src_h_u as f64;

    for dy in 0..dst_h {
        let ry = dy as f64 - dst_cy;
        // Base source coordinates at dx=0 for this row
        let sx_base = -dst_cx * cos_inv - ry * sin_inv + cx;
        let sy_base = -dst_cx * sin_inv + ry * cos_inv + cy;

        for dx in 0..dst_w {
            // Incremental update: sx += cos_inv, sy += sin_inv per dx step
            let sx = sx_base + dx as f64 * cos_inv;
            let sy = sy_base + dx as f64 * sin_inv;

            // Bilinear interpolation (inclusive of last row/column)
            if sx >= 0.0 && sy >= 0.0 && sx < src_w_f && sy < src_h_f {
                let x0 = sx.floor() as u32;
                let y0 = sy.floor() as u32;
                let x1 = (x0 + 1).min(src_w_u - 1);
                let y1 = (y0 + 1).min(src_h_u - 1);

                let fx = sx - sx.floor();
                let fy = sy - sy.floor();

                // Precompute bilinear weights once per pixel
                let w00 = (1.0 - fx) * (1.0 - fy);
                let w10 = fx * (1.0 - fy);
                let w01 = (1.0 - fx) * fy;
                let w11 = fx * fy;

                let p00 = src.get_pixel(x0, y0).0;
                let p10 = src.get_pixel(x1, y0).0;
                let p01 = src.get_pixel(x0, y1).0;
                let p11 = src.get_pixel(x1, y1).0;

                let mut pixel = [0u8; 4];
                for c in 0..4 {
                    let v = p00[c] as f64 * w00
                        + p10[c] as f64 * w10
                        + p01[c] as f64 * w01
                        + p11[c] as f64 * w11;
                    pixel[c] = v.round().clamp(0.0, 255.0) as u8;
                }
                dst.put_pixel(dx, dy, Rgba(pixel));
            }
            // else: leave as background
        }
    }

    DynamicImage::ImageRgba8(dst)
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
            RotateAngle::Arbitrary(deg) => {
                const ANGLE_TOL: f64 = 1e-9;
                let normalized = ((deg % 360.0) + 360.0) % 360.0;
                if normalized.abs() < ANGLE_TOL || (normalized - 360.0).abs() < ANGLE_TOL {
                    img // no-op
                } else {
                    rotate_arbitrary(&img, deg, self.background)
                }
            }
        })
    }

    fn describe(&self) -> OperationDescription {
        let deg = self.angle.degrees_f64();
        let mut params = serde_json::json!({ "angle": deg });
        if !self.angle.is_cardinal() {
            let bg = self.background;
            params["background"] = serde_json::json!(format!(
                "#{:02X}{:02X}{:02X}{:02X}",
                bg[0], bg[1], bg[2], bg[3]
            ));
        }
        OperationDescription {
            operation: "rotate".into(),
            params,
            description: format!("Rotate {deg} degrees clockwise"),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "rotate".into(),
            description: "Rotate an image by any angle (90/180/270 use fast lossless path)".into(),
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
                    description: "Rotation angle in degrees (e.g. 90, 180, 270, 45, 30.5) or aliases: left, right".into(),
                    default: None,
                    choices: None,
                    range: None,
                },
                ParamSchema {
                    name: "background".into(),
                    param_type: ParamType::String,
                    required: false,
                    description: "Background fill color for arbitrary angles (hex, RGB, or named). Default: transparent".into(),
                    default: Some("transparent".into()),
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

    fn test_image_rgb(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgb8(image::RgbImage::from_fn(w, h, |_, _| {
            image::Rgb([128, 128, 128])
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
    }

    #[test]
    fn parse_arbitrary_angle() {
        assert!(matches!(
            RotateAngle::parse("45"),
            Ok(RotateAngle::Arbitrary(a)) if (a - 45.0).abs() < f64::EPSILON
        ));
        assert!(matches!(
            RotateAngle::parse("30.5"),
            Ok(RotateAngle::Arbitrary(a)) if (a - 30.5).abs() < f64::EPSILON
        ));
        assert!(matches!(
            RotateAngle::parse("-45"),
            Ok(RotateAngle::Arbitrary(a)) if (a - -45.0).abs() < f64::EPSILON
        ));
    }

    #[test]
    fn parse_float_cardinal_angles_use_fast_path() {
        // "90.0" as float should still map to the fast path
        assert!(matches!(RotateAngle::parse("90.0"), Ok(RotateAngle::Deg90)));
        assert!(matches!(
            RotateAngle::parse("180.0"),
            Ok(RotateAngle::Deg180)
        ));
        assert!(matches!(
            RotateAngle::parse("270.0"),
            Ok(RotateAngle::Deg270)
        ));
    }

    #[test]
    fn parse_zero_angle() {
        assert!(matches!(
            RotateAngle::parse("0"),
            Ok(RotateAngle::Arbitrary(a)) if a.abs() < f64::EPSILON
        ));
    }

    #[test]
    fn parse_invalid_angle() {
        assert!(RotateAngle::parse("abc").is_err());
    }

    #[test]
    fn rotate_arbitrary_45_degrees() {
        let img = test_image(100, 100);
        let op = RotateOp::new(RotateAngle::Arbitrary(45.0));
        let result = op.apply(img).unwrap();
        // A 100x100 image rotated 45° should produce a larger bounding box
        // Expected: ~141x141 (100*sqrt(2))
        assert!(result.width() > 100);
        assert!(result.height() > 100);
        let expected = (100.0 * std::f64::consts::SQRT_2).ceil() as u32;
        assert!(
            (result.width() as i32 - expected as i32).unsigned_abs() <= 2,
            "width {} not close to expected {}",
            result.width(),
            expected
        );
    }

    #[test]
    fn rotate_arbitrary_zero_is_noop() {
        let img = test_image(100, 50);
        let op = RotateOp::new(RotateAngle::Arbitrary(0.0));
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 100);
        assert_eq!(result.height(), 50);
    }

    #[test]
    fn rotate_arbitrary_with_background_color() {
        let img = test_image(100, 100);
        let bg = Rgba([255, 0, 0, 255]);
        let op = RotateOp::new(RotateAngle::Arbitrary(45.0)).with_background(bg);
        let result = op.apply(img).unwrap();
        // Corner pixel should be the background color (red)
        let corner = result.to_rgba8().get_pixel(0, 0).0;
        assert_eq!(corner, [255, 0, 0, 255]);
    }

    #[test]
    fn rotate_arbitrary_rgb_image() {
        let img = test_image_rgb(80, 60);
        let op =
            RotateOp::new(RotateAngle::Arbitrary(30.0)).with_background(Rgba([255, 255, 255, 255]));
        let result = op.apply(img).unwrap();
        // Should succeed and produce a valid image
        assert!(result.width() > 0);
        assert!(result.height() > 0);
    }

    #[test]
    fn rotate_arbitrary_negative_angle() {
        let img = test_image(100, 100);
        let op = RotateOp::new(RotateAngle::Arbitrary(-30.0));
        let result = op.apply(img).unwrap();
        assert!(result.width() > 100);
        assert!(result.height() > 100);
    }

    #[test]
    fn degrees_f64_returns_correct_values() {
        assert!((RotateAngle::Deg90.degrees_f64() - 90.0).abs() < f64::EPSILON);
        assert!((RotateAngle::Deg180.degrees_f64() - 180.0).abs() < f64::EPSILON);
        assert!((RotateAngle::Deg270.degrees_f64() - 270.0).abs() < f64::EPSILON);
        assert!((RotateAngle::Arbitrary(45.5).degrees_f64() - 45.5).abs() < f64::EPSILON);
    }

    #[test]
    fn is_cardinal_correct() {
        assert!(RotateAngle::Deg90.is_cardinal());
        assert!(RotateAngle::Deg180.is_cardinal());
        assert!(RotateAngle::Deg270.is_cardinal());
        assert!(!RotateAngle::Arbitrary(45.0).is_cardinal());
    }
}
