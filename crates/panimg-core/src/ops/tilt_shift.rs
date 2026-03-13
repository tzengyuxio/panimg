use crate::error::{PanimgError, Result};
use crate::ops::color::{hsl_to_rgb, rgb_to_hsl};
use crate::ops::{Operation, OperationDescription};
use crate::schema::{CommandSchema, ParamRange, ParamSchema, ParamType};
use image::DynamicImage;

/// Tilt-shift (miniature/diorama) effect.
///
/// Keeps a horizontal focus band sharp and progressively blurs the rest,
/// optionally boosting saturation for a toy-model look.
pub struct TiltShiftOp {
    pub sigma: f32,
    pub focus_position: f32,
    pub focus_width: f32,
    pub transition: f32,
    pub saturation: f32,
}

impl TiltShiftOp {
    pub fn new(
        sigma: f32,
        focus_position: f32,
        focus_width: f32,
        transition: f32,
        saturation: f32,
    ) -> Result<Self> {
        if !(0.1..=100.0).contains(&sigma) {
            return Err(PanimgError::InvalidArgument {
                message: format!("sigma must be between 0.1 and 100.0, got {sigma}"),
                suggestion: "use a value like 8.0 or 12.0".into(),
            });
        }
        if !(0.0..=1.0).contains(&focus_position) {
            return Err(PanimgError::InvalidArgument {
                message: format!(
                    "focus-position must be between 0.0 and 1.0, got {focus_position}"
                ),
                suggestion: "0.5 = center, 0.3 = upper third, 0.7 = lower third".into(),
            });
        }
        if !(0.01..=1.0).contains(&focus_width) {
            return Err(PanimgError::InvalidArgument {
                message: format!("focus-width must be between 0.01 and 1.0, got {focus_width}"),
                suggestion: "use a value like 0.15 (15% of image height)".into(),
            });
        }
        if !(0.01..=1.0).contains(&transition) {
            return Err(PanimgError::InvalidArgument {
                message: format!("transition must be between 0.01 and 1.0, got {transition}"),
                suggestion: "use a value like 0.2 (20% of image height)".into(),
            });
        }
        if !(0.0..=3.0).contains(&saturation) {
            return Err(PanimgError::InvalidArgument {
                message: format!("saturation must be between 0.0 and 3.0, got {saturation}"),
                suggestion: "1.0 = unchanged, 1.3 = slightly boosted for miniature look".into(),
            });
        }
        Ok(Self {
            sigma,
            focus_position,
            focus_width,
            transition,
            saturation,
        })
    }
}

/// Generate a per-row blur mask.
///
/// Returns a `Vec<f32>` of length `h` where each value is in `[0.0, 1.0]`:
/// - `0.0` = keep original (in-focus)
/// - `1.0` = fully blurred (out-of-focus)
///
/// The mask has three zones per side of the focus band:
///   focus band (t=0) → transition (smoothstep 0→1) → fully blurred (t=1)
fn generate_mask(h: u32, focus_position: f32, focus_width: f32, transition: f32) -> Vec<f32> {
    let hf = h as f32;
    let center = focus_position * hf;
    let half_band = focus_width * hf * 0.5;
    let trans_pixels = transition * hf;

    (0..h)
        .map(|row| {
            let y = row as f32 + 0.5;
            let dist = (y - center).abs() - half_band;
            if dist <= 0.0 {
                0.0 // inside focus band
            } else if dist >= trans_pixels {
                1.0 // fully blurred
            } else {
                // smoothstep interpolation: 3t² - 2t³
                let t = dist / trans_pixels;
                t * t * (3.0 - 2.0 * t)
            }
        })
        .collect()
}

impl Operation for TiltShiftOp {
    fn name(&self) -> &str {
        "tilt-shift"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        let (w, h) = (img.width(), img.height());
        let blurred = img.blur(self.sigma);

        let mask = generate_mask(h, self.focus_position, self.focus_width, self.transition);

        let mut out = img.to_rgba8();
        let blur_rgba = blurred.to_rgba8();
        let apply_sat = (self.saturation - 1.0).abs() > f32::EPSILON;

        for y in 0..h {
            let t = mask[y as usize];
            for x in 0..w {
                let need_blend = t > 0.0;
                let need_work = need_blend || apply_sat;
                if !need_work {
                    continue;
                }

                let op = out.get_pixel(x, y);
                let (mut r, mut g, mut b) = (op[0], op[1], op[2]);

                if need_blend {
                    let bp = blur_rgba.get_pixel(x, y);
                    let inv_t = 1.0 - t;
                    r = (r as f32 * inv_t + bp[0] as f32 * t).round() as u8;
                    g = (g as f32 * inv_t + bp[1] as f32 * t).round() as u8;
                    b = (b as f32 * inv_t + bp[2] as f32 * t).round() as u8;
                }

                if apply_sat {
                    let (h, s, l) = rgb_to_hsl(r, g, b);
                    let new_s = (s * self.saturation).clamp(0.0, 1.0);
                    (r, g, b) = hsl_to_rgb(h, new_s, l);
                }

                out.put_pixel(x, y, image::Rgba([r, g, b, op[3]]));
            }
        }

        Ok(DynamicImage::ImageRgba8(out))
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "tilt-shift".into(),
            params: serde_json::json!({
                "sigma": self.sigma,
                "focus_position": self.focus_position,
                "focus_width": self.focus_width,
                "transition": self.transition,
                "saturation": self.saturation,
            }),
            description: format!(
                "Tilt-shift effect (sigma={}, focus={:.0}%, width={:.0}%, transition={:.0}%, saturation={})",
                self.sigma,
                self.focus_position * 100.0,
                self.focus_width * 100.0,
                self.transition * 100.0,
                self.saturation,
            ),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "tilt-shift".into(),
            description: "Simulate tilt-shift (miniature/diorama) lens effect".into(),
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
                    name: "sigma".into(),
                    param_type: ParamType::Float,
                    required: false,
                    description: "Out-of-focus blur strength (default: 8.0)".into(),
                    default: Some(serde_json::json!(8.0)),
                    choices: None,
                    range: Some(ParamRange {
                        min: 0.1,
                        max: 100.0,
                    }),
                },
                ParamSchema {
                    name: "focus_position".into(),
                    param_type: ParamType::Float,
                    required: false,
                    description:
                        "Vertical center of the focus band (0=top, 1=bottom, default: 0.5)".into(),
                    default: Some(serde_json::json!(0.5)),
                    choices: None,
                    range: Some(ParamRange { min: 0.0, max: 1.0 }),
                },
                ParamSchema {
                    name: "focus_width".into(),
                    param_type: ParamType::Float,
                    required: false,
                    description:
                        "Height of the focus band as fraction of image height (default: 0.15)"
                            .into(),
                    default: Some(serde_json::json!(0.15)),
                    choices: None,
                    range: Some(ParamRange {
                        min: 0.01,
                        max: 1.0,
                    }),
                },
                ParamSchema {
                    name: "transition".into(),
                    param_type: ParamType::Float,
                    required: false,
                    description: "Transition zone width as fraction of image height (default: 0.2)"
                        .into(),
                    default: Some(serde_json::json!(0.2)),
                    choices: None,
                    range: Some(ParamRange {
                        min: 0.01,
                        max: 1.0,
                    }),
                },
                ParamSchema {
                    name: "saturation".into(),
                    param_type: ParamType::Float,
                    required: false,
                    description: "Saturation multiplier (>1 enhances miniature look, default: 1.0)"
                        .into(),
                    default: Some(serde_json::json!(1.0)),
                    choices: None,
                    range: Some(ParamRange { min: 0.0, max: 3.0 }),
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, RgbaImage};

    fn test_image() -> DynamicImage {
        // 16x16 gradient image — top is dark, bottom is bright
        DynamicImage::ImageRgba8(RgbaImage::from_fn(16, 16, |_x, y| {
            let v = (y * 16).min(255) as u8;
            image::Rgba([v, v, v, 255])
        }))
    }

    #[test]
    fn preserves_dimensions() {
        let op = TiltShiftOp::new(4.0, 0.5, 0.15, 0.2, 1.0).unwrap();
        let result = op.apply(test_image()).unwrap();
        assert_eq!(result.width(), 16);
        assert_eq!(result.height(), 16);
    }

    #[test]
    fn focus_band_stays_sharp() {
        // With focus at center and a very wide focus band (100%), the entire
        // image should remain unblurred — output ≈ original.
        let op = TiltShiftOp::new(8.0, 0.5, 1.0, 0.01, 1.0).unwrap();
        let img = test_image();
        let orig = img.to_rgba8();
        let result = op.apply(img).unwrap().to_rgba8();

        for y in 0..16u32 {
            for x in 0..16u32 {
                let o = orig.get_pixel(x, y);
                let r = result.get_pixel(x, y);
                // Allow tiny rounding differences
                assert!(
                    (o[0] as i32 - r[0] as i32).unsigned_abs() <= 1,
                    "pixel ({x},{y}) diverged: orig={}, result={}",
                    o[0],
                    r[0]
                );
            }
        }
    }

    #[test]
    fn out_of_focus_region_is_blurred() {
        // Focus band at the very top (position=0, tiny width), large sigma.
        // Bottom rows should differ from original (blurred).
        let op = TiltShiftOp::new(10.0, 0.0, 0.01, 0.01, 1.0).unwrap();
        let img = test_image();
        let orig = img.to_rgba8();
        let result = op.apply(img).unwrap().to_rgba8();

        // Check a bottom row
        let y = 15u32;
        let mut any_diff = false;
        for x in 0..16u32 {
            let o = orig.get_pixel(x, y);
            let r = result.get_pixel(x, y);
            if o[0] != r[0] {
                any_diff = true;
                break;
            }
        }
        assert!(
            any_diff,
            "bottom row should be blurred and differ from original"
        );
    }

    #[test]
    fn saturation_boost_changes_colors() {
        // Create a colored image and apply saturation > 1
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(
            4,
            4,
            image::Rgba([200, 100, 50, 255]),
        ));
        let op = TiltShiftOp::new(2.0, 0.5, 0.5, 0.2, 2.0).unwrap();
        let result = op.apply(img).unwrap().to_rgba8();
        let p = result.get_pixel(2, 2);
        // With boosted saturation, the dominant channel (red) should be even more dominant
        assert!(p[0] > p[1]);
        assert!(p[1] > p[2]);
    }

    #[test]
    fn generate_mask_center_focus() {
        let mask = generate_mask(100, 0.5, 0.2, 0.1);
        assert_eq!(mask.len(), 100);
        // Center rows should be 0 (in-focus)
        assert_eq!(mask[50], 0.0);
        assert_eq!(mask[45], 0.0);
        assert_eq!(mask[55], 0.0);
        // Edge rows should be 1.0 (fully blurred)
        assert_eq!(mask[0], 1.0);
        assert_eq!(mask[99], 1.0);
    }

    #[test]
    fn generate_mask_smoothstep_monotonic() {
        let mask = generate_mask(200, 0.5, 0.1, 0.3);
        // From center outward (going down), mask values should be non-decreasing
        for i in 100..199 {
            assert!(
                mask[i + 1] >= mask[i] - f32::EPSILON,
                "mask not monotonic at row {i}: {} > {}",
                mask[i],
                mask[i + 1]
            );
        }
    }

    #[test]
    fn invalid_params() {
        assert!(TiltShiftOp::new(0.0, 0.5, 0.15, 0.2, 1.0).is_err()); // sigma too low
        assert!(TiltShiftOp::new(101.0, 0.5, 0.15, 0.2, 1.0).is_err()); // sigma too high
        assert!(TiltShiftOp::new(8.0, -0.1, 0.15, 0.2, 1.0).is_err()); // focus_position < 0
        assert!(TiltShiftOp::new(8.0, 1.1, 0.15, 0.2, 1.0).is_err()); // focus_position > 1
        assert!(TiltShiftOp::new(8.0, 0.5, 0.0, 0.2, 1.0).is_err()); // focus_width too small
        assert!(TiltShiftOp::new(8.0, 0.5, 0.15, 0.0, 1.0).is_err()); // transition too small
        assert!(TiltShiftOp::new(8.0, 0.5, 0.15, 0.2, -0.1).is_err()); // saturation < 0
        assert!(TiltShiftOp::new(8.0, 0.5, 0.15, 0.2, 3.1).is_err()); // saturation > 3
    }
}
