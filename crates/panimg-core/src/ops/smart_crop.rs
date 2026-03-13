use crate::error::{PanimgError, Result};
use crate::ops::color::rgb_to_hsl;
use crate::ops::crop::CropOp;
use crate::ops::{Operation, OperationDescription};
use crate::schema::{CommandSchema, ParamRange, ParamSchema, ParamType};
use image::DynamicImage;
use std::fmt;

/// Strategy for evaluating crop regions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmartCropStrategy {
    /// Shannon entropy — prefers regions with the most information.
    Entropy,
    /// Weighted combination of edges, saturation, and skin tones.
    Attention,
}

impl SmartCropStrategy {
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "entropy" => Ok(Self::Entropy),
            "attention" => Ok(Self::Attention),
            _ => Err(PanimgError::InvalidArgument {
                message: format!("unknown smart-crop strategy: '{s}'"),
                suggestion: "use 'entropy' or 'attention'".into(),
            }),
        }
    }
}

impl fmt::Display for SmartCropStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Entropy => f.write_str("entropy"),
            Self::Attention => f.write_str("attention"),
        }
    }
}

/// Smart-crop operation: automatically select the best crop region.
pub struct SmartCropOp {
    pub width: u32,
    pub height: u32,
    pub strategy: SmartCropStrategy,
    pub step: Option<u32>,
}

impl SmartCropOp {
    pub fn new(
        width: u32,
        height: u32,
        strategy: SmartCropStrategy,
        step: Option<u32>,
    ) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(PanimgError::InvalidArgument {
                message: "crop width and height must be greater than 0".into(),
                suggestion: "specify positive --width and --height values".into(),
            });
        }
        Ok(Self {
            width,
            height,
            strategy,
            step,
        })
    }

    /// Find the best crop origin (x, y) for the given image.
    pub fn find_best_crop(&self, img: &DynamicImage) -> Result<(u32, u32)> {
        let (img_w, img_h) = (img.width(), img.height());

        if self.width > img_w || self.height > img_h {
            return Err(PanimgError::InvalidArgument {
                message: format!(
                    "crop size {}x{} exceeds image dimensions {}x{}",
                    self.width, self.height, img_w, img_h
                ),
                suggestion: "use a smaller crop size or a larger image".into(),
            });
        }

        // If crop == image size, only one position possible.
        if self.width == img_w && self.height == img_h {
            return Ok((0, 0));
        }

        let range_x = img_w - self.width;
        let range_y = img_h - self.height;

        // Large-image optimisation: work on a downscaled copy.
        let max_dim = img_w.max(img_h);
        let (work_img, scale) = if max_dim > 2000 {
            let s = 1000.0 / max_dim as f64;
            let sw = ((img_w as f64 * s).round() as u32).max(1);
            let sh = ((img_h as f64 * s).round() as u32).max(1);
            (
                Some(img.resize_exact(sw, sh, image::imageops::FilterType::Triangle)),
                s,
            )
        } else {
            (None, 1.0)
        };
        let work_ref = work_img.as_ref().unwrap_or(img);

        let crop_w = ((self.width as f64 * scale).round() as u32).max(1);
        let crop_h = ((self.height as f64 * scale).round() as u32).max(1);
        let work_w = work_ref.width();
        let work_h = work_ref.height();
        let wr_x = work_w.saturating_sub(crop_w);
        let wr_y = work_h.saturating_sub(crop_h);

        let auto_step = (wr_x.max(wr_y) / 50).max(1);
        let step = self.step.map(|s| s.max(1)).unwrap_or(auto_step);

        // Pre-convert image data once outside the search loop.
        let best_pos = match self.strategy {
            SmartCropStrategy::Entropy => {
                let luma = work_ref.to_luma8();
                search_grid(wr_x, wr_y, step, |x, y| {
                    score_entropy(&luma, x, y, crop_w, crop_h)
                })
            }
            SmartCropStrategy::Attention => {
                let edge_map = compute_edge_map(work_ref);
                let rgba = work_ref.to_rgba8();
                search_grid(wr_x, wr_y, step, |x, y| {
                    score_attention(&rgba, &edge_map, work_w, x, y, crop_w, crop_h)
                })
            }
        };

        // Map back to original coordinates.
        let orig_x = ((best_pos.0 as f64 / scale).round() as u32).min(range_x);
        let orig_y = ((best_pos.1 as f64 / scale).round() as u32).min(range_y);

        Ok((orig_x, orig_y))
    }
}

/// Advance within a bounded range, ensuring the boundary is always visited.
fn step_next(current: u32, step: u32, max: u32) -> u32 {
    if current + step > max && current < max {
        max
    } else {
        current + step
    }
}

/// Slide a window over a 2D grid and return the position with the highest score.
fn search_grid(wr_x: u32, wr_y: u32, step: u32, score_fn: impl Fn(u32, u32) -> f64) -> (u32, u32) {
    let mut best_score = f64::NEG_INFINITY;
    let mut best_pos = (0u32, 0u32);

    let mut y = 0u32;
    while y <= wr_y {
        let mut x = 0u32;
        while x <= wr_x {
            let s = score_fn(x, y);
            if s > best_score {
                best_score = s;
                best_pos = (x, y);
            }
            x = step_next(x, step, wr_x);
        }
        y = step_next(y, step, wr_y);
    }
    best_pos
}

// --- Entropy scoring ---

fn score_entropy(luma: &image::GrayImage, x: u32, y: u32, w: u32, h: u32) -> f64 {
    let mut hist = [0u32; 256];
    let total = (w as u64) * (h as u64);

    for row in y..y + h {
        for col in x..x + w {
            let v = luma.get_pixel(col, row)[0];
            hist[v as usize] += 1;
        }
    }

    let mut entropy = 0.0f64;
    let total_f = total as f64;
    for &count in &hist {
        if count > 0 {
            let p = count as f64 / total_f;
            entropy -= p * p.log2();
        }
    }
    entropy
}

// --- Attention scoring ---

/// Compute a Laplacian edge magnitude map (grayscale convolution).
fn compute_edge_map(img: &DynamicImage) -> Vec<f32> {
    let luma = img.to_luma8();
    let (w, h) = (luma.width(), luma.height());
    let mut out = vec![0.0f32; (w * h) as usize];

    // Laplacian kernel: 0 -1 0 / -1 4 -1 / 0 -1 0
    for y in 1..h.saturating_sub(1) {
        for x in 1..w.saturating_sub(1) {
            let c = luma.get_pixel(x, y)[0] as f32;
            let t = luma.get_pixel(x, y - 1)[0] as f32;
            let b = luma.get_pixel(x, y + 1)[0] as f32;
            let l = luma.get_pixel(x - 1, y)[0] as f32;
            let r = luma.get_pixel(x + 1, y)[0] as f32;
            let val = (4.0 * c - t - b - l - r).abs();
            out[(y * w + x) as usize] = val;
        }
    }
    out
}

fn score_attention(
    rgba: &image::RgbaImage,
    edge_map: &[f32],
    edge_w: u32,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) -> f64 {
    let total = (w as f64) * (h as f64);
    let mut edge_sum = 0.0f64;
    let mut sat_sum = 0.0f64;
    let mut skin_sum = 0.0f64;

    for row in y..y + h {
        for col in x..x + w {
            // Edge component
            edge_sum += edge_map[(row * edge_w + col) as usize] as f64;

            // Saturation + skin tone from HSL
            let px = rgba.get_pixel(col, row);
            let (h_val, s, _l) = rgb_to_hsl(px[0], px[1], px[2]);

            sat_sum += s as f64;

            // Skin tone: hue ~15°-45° and saturation > 0.2
            let hue_deg = h_val * 360.0;
            if (15.0..=45.0).contains(&hue_deg) && s > 0.2 {
                skin_sum += 1.0;
            }
        }
    }

    // Normalise each component to 0..1 range and combine.
    // Max edge value per pixel is ~1020, saturation is 0..1, skin is 0 or 1.
    let edge_norm = edge_sum / (total * 1020.0);
    let sat_norm = sat_sum / total;
    let skin_norm = skin_sum / total;

    0.5 * edge_norm + 0.3 * sat_norm + 0.2 * skin_norm
}

impl Operation<DynamicImage, PanimgError> for SmartCropOp {
    fn name(&self) -> &str {
        "smart-crop"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        let (x, y) = self.find_best_crop(&img)?;
        let crop_op = CropOp::new(x, y, self.width, self.height)?;
        crop_op.apply(img)
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "smart-crop".into(),
            params: serde_json::json!({
                "width": self.width,
                "height": self.height,
                "strategy": self.strategy.to_string(),
                "step": self.step,
            }),
            description: format!(
                "Smart-crop {}x{} using {} strategy",
                self.width, self.height, self.strategy,
            ),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "smart-crop".into(),
            description: "Automatically select the best crop region based on image content".into(),
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
                    name: "width".into(),
                    param_type: ParamType::Integer,
                    required: true,
                    description: "Crop width in pixels".into(),
                    default: None,
                    choices: None,
                    range: Some(ParamRange {
                        min: 1.0,
                        max: 65535.0,
                    }),
                },
                ParamSchema {
                    name: "height".into(),
                    param_type: ParamType::Integer,
                    required: true,
                    description: "Crop height in pixels".into(),
                    default: None,
                    choices: None,
                    range: Some(ParamRange {
                        min: 1.0,
                        max: 65535.0,
                    }),
                },
                ParamSchema {
                    name: "strategy".into(),
                    param_type: ParamType::String,
                    required: false,
                    description: "Scoring strategy: entropy or attention (default: entropy)".into(),
                    default: Some(serde_json::json!("entropy")),
                    choices: Some(vec!["entropy".into(), "attention".into()]),
                    range: None,
                },
                ParamSchema {
                    name: "step".into(),
                    param_type: ParamType::Integer,
                    required: false,
                    description: "Search step size in pixels (default: auto)".into(),
                    default: None,
                    choices: None,
                    range: Some(ParamRange {
                        min: 1.0,
                        max: 1000.0,
                    }),
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, RgbaImage};

    fn solid_image(w: u32, h: u32, color: [u8; 4]) -> DynamicImage {
        DynamicImage::ImageRgba8(RgbaImage::from_pixel(w, h, image::Rgba(color)))
    }

    #[test]
    fn preserves_target_dimensions() {
        // Create a 100x100 image, crop to 50x50.
        let img = DynamicImage::ImageRgba8(RgbaImage::from_fn(100, 100, |x, y| {
            image::Rgba([x as u8, y as u8, 128, 255])
        }));
        let op = SmartCropOp::new(50, 50, SmartCropStrategy::Entropy, None).unwrap();
        let result = op.apply(img).unwrap();
        assert_eq!(result.width(), 50);
        assert_eq!(result.height(), 50);
    }

    #[test]
    fn crop_equals_image_when_same_size() {
        let img = solid_image(80, 60, [100, 150, 200, 255]);
        let op = SmartCropOp::new(80, 60, SmartCropStrategy::Entropy, None).unwrap();
        let (x, y) = op.find_best_crop(&img).unwrap();
        assert_eq!((x, y), (0, 0));
    }

    #[test]
    fn crop_larger_than_image_rejected() {
        let img = solid_image(50, 50, [128, 128, 128, 255]);
        let op = SmartCropOp::new(100, 100, SmartCropStrategy::Entropy, None).unwrap();
        assert!(op.find_best_crop(&img).is_err());
    }

    #[test]
    fn zero_dimensions_rejected() {
        assert!(SmartCropOp::new(0, 50, SmartCropStrategy::Entropy, None).is_err());
        assert!(SmartCropOp::new(50, 0, SmartCropStrategy::Entropy, None).is_err());
    }

    #[test]
    fn entropy_prefers_detailed_region() {
        // Left half: random noise (high entropy), right half: solid color (low entropy).
        let img = DynamicImage::ImageRgba8(RgbaImage::from_fn(100, 50, |x, y| {
            if x < 50 {
                // Pseudo-random noise using simple hash
                let v = ((x * 73 + y * 137 + 43) % 256) as u8;
                image::Rgba([v, v.wrapping_add(50), v.wrapping_add(100), 255])
            } else {
                image::Rgba([128, 128, 128, 255])
            }
        }));
        let op = SmartCropOp::new(40, 50, SmartCropStrategy::Entropy, Some(1)).unwrap();
        let (x, _y) = op.find_best_crop(&img).unwrap();
        // Should prefer the noisy left side.
        assert!(x < 30, "expected crop x < 30 (noisy region), got {x}");
    }

    #[test]
    fn attention_prefers_edges() {
        // Left half: strong vertical edge (black/white boundary), right half: uniform gray.
        let img = DynamicImage::ImageRgba8(RgbaImage::from_fn(100, 50, |x, _y| {
            if x < 50 {
                if x < 25 {
                    image::Rgba([0, 0, 0, 255])
                } else {
                    image::Rgba([255, 255, 255, 255])
                }
            } else {
                image::Rgba([128, 128, 128, 255])
            }
        }));
        let op = SmartCropOp::new(40, 50, SmartCropStrategy::Attention, Some(1)).unwrap();
        let (x, _y) = op.find_best_crop(&img).unwrap();
        // Should prefer the left side containing edges.
        assert!(x < 30, "expected crop x < 30 (edge region), got {x}");
    }

    #[test]
    fn custom_step_works() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_fn(100, 100, |x, y| {
            let v = ((x + y) % 256) as u8;
            image::Rgba([v, v, v, 255])
        }));
        let op = SmartCropOp::new(50, 50, SmartCropStrategy::Entropy, Some(10)).unwrap();
        let result = op.apply(img);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert_eq!(r.width(), 50);
        assert_eq!(r.height(), 50);
    }

    #[test]
    fn invalid_strategy_rejected() {
        assert!(SmartCropStrategy::parse("bogus").is_err());
        assert!(SmartCropStrategy::parse("").is_err());
    }

    #[test]
    fn find_best_crop_returns_valid_coords() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_fn(200, 150, |x, y| {
            let v = ((x * 7 + y * 13) % 256) as u8;
            image::Rgba([v, v, v, 255])
        }));
        let op = SmartCropOp::new(80, 60, SmartCropStrategy::Entropy, None).unwrap();
        let (x, y) = op.find_best_crop(&img).unwrap();
        assert!(x + 80 <= 200, "x={x} would exceed image width");
        assert!(y + 60 <= 150, "y={y} would exceed image height");
    }
}
