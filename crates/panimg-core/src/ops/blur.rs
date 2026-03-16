use crate::error::{PanimgError, Result};
use crate::ops::{Operation, OperationDescription};
use crate::schema::{CommandSchema, ParamRange, ParamSchema, ParamType};
use image::DynamicImage;

fn validate_sigma(sigma: f32) -> Result<()> {
    if sigma <= 0.0 {
        return Err(PanimgError::InvalidArgument {
            message: format!("blur sigma must be positive, got {sigma}"),
            suggestion: "use a value like 1.0, 2.5, or 5.0".into(),
        });
    }
    if sigma > 100.0 {
        return Err(PanimgError::InvalidArgument {
            message: format!("blur sigma {sigma} is too large"),
            suggestion: "use a value between 0.1 and 100.0".into(),
        });
    }
    Ok(())
}

fn validate_kernel(radius: u32, max: u32) -> Result<()> {
    if radius == 0 {
        return Err(PanimgError::InvalidArgument {
            message: "kernel radius must be at least 1".into(),
            suggestion: "use a value like 1, 2, or 3".into(),
        });
    }
    if radius > max {
        return Err(PanimgError::InvalidArgument {
            message: format!("kernel radius {radius} is too large"),
            suggestion: format!("use a value between 1 and {max}"),
        });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Gaussian blur (existing)
// ---------------------------------------------------------------------------

/// Gaussian blur operation.
pub struct BlurOp {
    /// Blur sigma (radius). Higher values = more blur.
    pub sigma: f32,
}

impl BlurOp {
    pub fn new(sigma: f32) -> Result<Self> {
        validate_sigma(sigma)?;
        Ok(Self { sigma })
    }
}

impl Operation<DynamicImage, PanimgError> for BlurOp {
    fn name(&self) -> &str {
        "blur"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        Ok(img.blur(self.sigma))
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "blur".into(),
            params: serde_json::json!({ "method": "gaussian", "sigma": self.sigma }),
            description: format!("Gaussian blur (sigma={})", self.sigma),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "blur".into(),
            description: "Apply blur to an image".into(),
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
                    name: "method".into(),
                    param_type: ParamType::String,
                    required: false,
                    description: "Blur method".into(),
                    default: Some(serde_json::json!("gaussian")),
                    choices: Some(vec![
                        "gaussian".into(),
                        "box".into(),
                        "motion".into(),
                        "median".into(),
                        "bilateral".into(),
                    ]),
                    range: None,
                },
                ParamSchema {
                    name: "sigma".into(),
                    param_type: ParamType::Float,
                    required: false,
                    description: "Blur radius (sigma). For gaussian and bilateral".into(),
                    default: None,
                    choices: None,
                    range: Some(ParamRange {
                        min: 0.1,
                        max: 100.0,
                    }),
                },
                ParamSchema {
                    name: "radius".into(),
                    param_type: ParamType::Integer,
                    required: false,
                    description: "Kernel radius. For box, median, and bilateral".into(),
                    default: None,
                    choices: None,
                    range: Some(ParamRange {
                        min: 1.0,
                        max: 50.0,
                    }),
                },
                ParamSchema {
                    name: "angle".into(),
                    param_type: ParamType::Float,
                    required: false,
                    description: "Motion blur angle in degrees (0=horizontal)".into(),
                    default: None,
                    choices: None,
                    range: Some(ParamRange {
                        min: 0.0,
                        max: 360.0,
                    }),
                },
                ParamSchema {
                    name: "distance".into(),
                    param_type: ParamType::Integer,
                    required: false,
                    description: "Motion blur distance in pixels".into(),
                    default: None,
                    choices: None,
                    range: Some(ParamRange {
                        min: 1.0,
                        max: 200.0,
                    }),
                },
                ParamSchema {
                    name: "sigma_color".into(),
                    param_type: ParamType::Float,
                    required: false,
                    description: "Bilateral filter color sigma (intensity similarity)".into(),
                    default: None,
                    choices: None,
                    range: Some(ParamRange {
                        min: 0.1,
                        max: 255.0,
                    }),
                },
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// Box blur
// ---------------------------------------------------------------------------

/// Box (mean) blur — fast uniform filter.
pub struct BoxBlurOp {
    pub radius: u32,
}

impl BoxBlurOp {
    pub fn new(radius: u32) -> Result<Self> {
        validate_kernel(radius, 50)?;
        Ok(Self { radius })
    }
}

impl Operation<DynamicImage, PanimgError> for BoxBlurOp {
    fn name(&self) -> &str {
        "blur"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        let rgba = img.to_rgba8();
        let (w, h) = (rgba.width(), rgba.height());
        let src = rgba.as_raw();
        let r = self.radius as i32;
        let kernel_size = (2 * r + 1) as f64;
        let area = kernel_size * kernel_size;

        let mut dst = vec![0u8; src.len()];

        for y in 0..h as i32 {
            for x in 0..w as i32 {
                let mut sum = [0.0f64; 4];
                for ky in -r..=r {
                    for kx in -r..=r {
                        let sx = (x + kx).clamp(0, w as i32 - 1) as u32;
                        let sy = (y + ky).clamp(0, h as i32 - 1) as u32;
                        let idx = ((sy * w + sx) * 4) as usize;
                        for c in 0..4 {
                            sum[c] += src[idx + c] as f64;
                        }
                    }
                }
                let idx = ((y as u32 * w + x as u32) * 4) as usize;
                for c in 0..4 {
                    dst[idx + c] = (sum[c] / area).round() as u8;
                }
            }
        }

        Ok(DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(w, h, dst).unwrap(),
        ))
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "blur".into(),
            params: serde_json::json!({ "method": "box", "radius": self.radius }),
            description: format!("Box blur (radius={})", self.radius),
        }
    }

    fn schema() -> CommandSchema {
        BlurOp::schema()
    }
}

// ---------------------------------------------------------------------------
// Motion blur
// ---------------------------------------------------------------------------

/// Motion blur — directional blur simulating camera/subject motion.
pub struct MotionBlurOp {
    /// Angle in degrees (0 = horizontal right).
    pub angle: f32,
    /// Length of the motion in pixels.
    pub distance: u32,
}

impl MotionBlurOp {
    pub fn new(angle: f32, distance: u32) -> Result<Self> {
        if distance == 0 {
            return Err(PanimgError::InvalidArgument {
                message: "motion blur distance must be at least 1".into(),
                suggestion: "use a value like 5, 10, or 20".into(),
            });
        }
        if distance > 200 {
            return Err(PanimgError::InvalidArgument {
                message: format!("motion blur distance {distance} is too large"),
                suggestion: "use a value between 1 and 200".into(),
            });
        }
        Ok(Self { angle, distance })
    }
}

impl Operation<DynamicImage, PanimgError> for MotionBlurOp {
    fn name(&self) -> &str {
        "blur"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        let rgba = img.to_rgba8();
        let (w, h) = (rgba.width(), rgba.height());
        let src = rgba.as_raw();
        let dist = self.distance as i32;
        let rad = self.angle.to_radians();
        let dx = rad.cos() as f64;
        let dy = rad.sin() as f64;

        let mut dst = vec![0u8; src.len()];

        for y in 0..h as i32 {
            for x in 0..w as i32 {
                let mut sum = [0.0f64; 4];
                let mut count = 0;
                for i in 0..dist {
                    let sx = (x as f64 + i as f64 * dx).round() as i32;
                    let sy = (y as f64 + i as f64 * dy).round() as i32;
                    let sx = sx.clamp(0, w as i32 - 1) as u32;
                    let sy = sy.clamp(0, h as i32 - 1) as u32;
                    let idx = ((sy * w + sx) * 4) as usize;
                    for c in 0..4 {
                        sum[c] += src[idx + c] as f64;
                    }
                    count += 1;
                }
                let idx = ((y as u32 * w + x as u32) * 4) as usize;
                for c in 0..4 {
                    dst[idx + c] = (sum[c] / count as f64).round() as u8;
                }
            }
        }

        Ok(DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(w, h, dst).unwrap(),
        ))
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "blur".into(),
            params: serde_json::json!({ "method": "motion", "angle": self.angle, "distance": self.distance }),
            description: format!(
                "Motion blur (angle={}°, distance={}px)",
                self.angle, self.distance
            ),
        }
    }

    fn schema() -> CommandSchema {
        BlurOp::schema()
    }
}

// ---------------------------------------------------------------------------
// Median blur
// ---------------------------------------------------------------------------

/// Median blur — effective for salt-and-pepper noise removal.
pub struct MedianBlurOp {
    pub radius: u32,
}

impl MedianBlurOp {
    pub fn new(radius: u32) -> Result<Self> {
        validate_kernel(radius, 10)?;
        Ok(Self { radius })
    }
}

impl Operation<DynamicImage, PanimgError> for MedianBlurOp {
    fn name(&self) -> &str {
        "blur"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        let rgba = img.to_rgba8();
        let (w, h) = (rgba.width(), rgba.height());
        let src = rgba.as_raw();
        let r = self.radius as i32;

        let mut dst = vec![0u8; src.len()];

        for y in 0..h as i32 {
            for x in 0..w as i32 {
                let mut channels: [Vec<u8>; 4] = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
                for ky in -r..=r {
                    for kx in -r..=r {
                        let sx = (x + kx).clamp(0, w as i32 - 1) as u32;
                        let sy = (y + ky).clamp(0, h as i32 - 1) as u32;
                        let idx = ((sy * w + sx) * 4) as usize;
                        for c in 0..4 {
                            channels[c].push(src[idx + c]);
                        }
                    }
                }
                let idx = ((y as u32 * w + x as u32) * 4) as usize;
                for c in 0..4 {
                    channels[c].sort_unstable();
                    dst[idx + c] = channels[c][channels[c].len() / 2];
                }
            }
        }

        Ok(DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(w, h, dst).unwrap(),
        ))
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "blur".into(),
            params: serde_json::json!({ "method": "median", "radius": self.radius }),
            description: format!("Median blur (radius={})", self.radius),
        }
    }

    fn schema() -> CommandSchema {
        BlurOp::schema()
    }
}

// ---------------------------------------------------------------------------
// Bilateral blur
// ---------------------------------------------------------------------------

/// Bilateral blur — edge-preserving smoothing.
pub struct BilateralBlurOp {
    /// Spatial extent (kernel radius in pixels).
    pub radius: u32,
    /// Spatial sigma (Gaussian weight for distance).
    pub sigma_spatial: f32,
    /// Color/intensity sigma (Gaussian weight for pixel similarity).
    pub sigma_color: f32,
}

impl BilateralBlurOp {
    pub fn new(radius: u32, sigma_spatial: f32, sigma_color: f32) -> Result<Self> {
        validate_kernel(radius, 20)?;
        validate_sigma(sigma_spatial)?;
        if sigma_color <= 0.0 {
            return Err(PanimgError::InvalidArgument {
                message: format!("sigma_color must be positive, got {sigma_color}"),
                suggestion: "use a value like 25.0, 50.0, or 75.0".into(),
            });
        }
        if sigma_color > 255.0 {
            return Err(PanimgError::InvalidArgument {
                message: format!("sigma_color {sigma_color} is too large"),
                suggestion: "use a value between 0.1 and 255.0".into(),
            });
        }
        Ok(Self {
            radius,
            sigma_spatial,
            sigma_color,
        })
    }
}

impl Operation<DynamicImage, PanimgError> for BilateralBlurOp {
    fn name(&self) -> &str {
        "blur"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        let rgba = img.to_rgba8();
        let (w, h) = (rgba.width(), rgba.height());
        let src = rgba.as_raw();
        let r = self.radius as i32;
        let spatial_denom = 2.0 * self.sigma_spatial * self.sigma_spatial;
        let color_denom = 2.0 * self.sigma_color * self.sigma_color;

        let mut dst = vec![0u8; src.len()];

        for y in 0..h as i32 {
            for x in 0..w as i32 {
                let center_idx = ((y as u32 * w + x as u32) * 4) as usize;
                let center = [
                    src[center_idx] as f64,
                    src[center_idx + 1] as f64,
                    src[center_idx + 2] as f64,
                ];

                let mut sum = [0.0f64; 3];
                let mut alpha_sum = 0.0f64;
                let mut weight_sum = 0.0f64;
                let mut alpha_weight_sum = 0.0f64;

                for ky in -r..=r {
                    for kx in -r..=r {
                        let sx = (x + kx).clamp(0, w as i32 - 1) as u32;
                        let sy = (y + ky).clamp(0, h as i32 - 1) as u32;
                        let idx = ((sy * w + sx) * 4) as usize;

                        let spatial_dist = (kx * kx + ky * ky) as f64;
                        let spatial_w = (-spatial_dist / spatial_denom as f64).exp();

                        let pixel = [src[idx] as f64, src[idx + 1] as f64, src[idx + 2] as f64];
                        let color_dist =
                            (0..3).map(|c| (pixel[c] - center[c]).powi(2)).sum::<f64>();
                        let color_w = (-color_dist / color_denom as f64).exp();

                        let w_total = spatial_w * color_w;
                        for c in 0..3 {
                            sum[c] += pixel[c] * w_total;
                        }
                        weight_sum += w_total;

                        // Alpha uses spatial weight only
                        alpha_sum += src[idx + 3] as f64 * spatial_w;
                        alpha_weight_sum += spatial_w;
                    }
                }

                for c in 0..3 {
                    dst[center_idx + c] = (sum[c] / weight_sum).round() as u8;
                }
                dst[center_idx + 3] = (alpha_sum / alpha_weight_sum).round() as u8;
            }
        }

        Ok(DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(w, h, dst).unwrap(),
        ))
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "blur".into(),
            params: serde_json::json!({
                "method": "bilateral",
                "radius": self.radius,
                "sigma_spatial": self.sigma_spatial,
                "sigma_color": self.sigma_color,
            }),
            description: format!(
                "Bilateral blur (radius={}, sigma_spatial={}, sigma_color={})",
                self.radius, self.sigma_spatial, self.sigma_color
            ),
        }
    }

    fn schema() -> CommandSchema {
        BlurOp::schema()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_image() -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::from_fn(8, 8, |x, y| {
            // Checkerboard pattern — blur should smooth it
            if (x + y) % 2 == 0 {
                image::Rgba([255, 255, 255, 255])
            } else {
                image::Rgba([0, 0, 0, 255])
            }
        }))
    }

    fn noisy_image() -> DynamicImage {
        // Solid gray with a few salt-and-pepper noise pixels
        DynamicImage::ImageRgba8(image::RgbaImage::from_fn(16, 16, |x, y| {
            if (x == 3 && y == 3) || (x == 10 && y == 7) {
                image::Rgba([255, 255, 255, 255]) // salt
            } else if (x == 5 && y == 12) || (x == 14 && y == 1) {
                image::Rgba([0, 0, 0, 255]) // pepper
            } else {
                image::Rgba([128, 128, 128, 255]) // gray
            }
        }))
    }

    // --- Gaussian blur ---

    #[test]
    fn blur_preserves_dimensions() {
        let op = BlurOp::new(1.0).unwrap();
        let result = op.apply(test_image()).unwrap();
        assert_eq!(result.width(), 8);
        assert_eq!(result.height(), 8);
    }

    #[test]
    fn blur_smooths_checkerboard() {
        let op = BlurOp::new(2.0).unwrap();
        let img = test_image();
        let result = op.apply(img).unwrap();
        let rgba = result.to_rgba8();
        // After blur, pixels should be somewhere between 0 and 255
        let p = rgba.get_pixel(4, 4);
        assert!(p[0] > 50 && p[0] < 200);
    }

    #[test]
    fn blur_invalid_sigma() {
        assert!(BlurOp::new(0.0).is_err());
        assert!(BlurOp::new(-1.0).is_err());
        assert!(BlurOp::new(101.0).is_err());
    }

    // --- Box blur ---

    #[test]
    fn box_blur_preserves_dimensions() {
        let op = BoxBlurOp::new(2).unwrap();
        let result = op.apply(test_image()).unwrap();
        assert_eq!(result.width(), 8);
        assert_eq!(result.height(), 8);
    }

    #[test]
    fn box_blur_smooths_checkerboard() {
        let op = BoxBlurOp::new(1).unwrap();
        let result = op.apply(test_image()).unwrap();
        let rgba = result.to_rgba8();
        let p = rgba.get_pixel(4, 4);
        assert!(p[0] > 50 && p[0] < 200);
    }

    #[test]
    fn box_blur_invalid_radius() {
        assert!(BoxBlurOp::new(0).is_err());
        assert!(BoxBlurOp::new(51).is_err());
    }

    // --- Motion blur ---

    #[test]
    fn motion_blur_preserves_dimensions() {
        let op = MotionBlurOp::new(0.0, 5).unwrap();
        let result = op.apply(test_image()).unwrap();
        assert_eq!(result.width(), 8);
        assert_eq!(result.height(), 8);
    }

    #[test]
    fn motion_blur_horizontal_smears() {
        // Horizontal motion on a checkerboard should create intermediate values
        let op = MotionBlurOp::new(0.0, 4).unwrap();
        let result = op.apply(test_image()).unwrap();
        let rgba = result.to_rgba8();
        let p = rgba.get_pixel(4, 4);
        assert!(p[0] > 50 && p[0] < 200);
    }

    #[test]
    fn motion_blur_invalid_distance() {
        assert!(MotionBlurOp::new(0.0, 0).is_err());
        assert!(MotionBlurOp::new(0.0, 201).is_err());
    }

    // --- Median blur ---

    #[test]
    fn median_blur_preserves_dimensions() {
        let op = MedianBlurOp::new(1).unwrap();
        let result = op.apply(test_image()).unwrap();
        assert_eq!(result.width(), 8);
        assert_eq!(result.height(), 8);
    }

    #[test]
    fn median_blur_removes_noise() {
        let op = MedianBlurOp::new(1).unwrap();
        let result = op.apply(noisy_image()).unwrap();
        let rgba = result.to_rgba8();
        // Noise pixels should have been replaced by median of neighbors (~128)
        let p = rgba.get_pixel(3, 3); // was salt (255)
        assert!(p[0] < 200, "salt pixel should be smoothed, got {}", p[0]);
    }

    #[test]
    fn median_blur_invalid_radius() {
        assert!(MedianBlurOp::new(0).is_err());
        assert!(MedianBlurOp::new(11).is_err());
    }

    // --- Bilateral blur ---

    #[test]
    fn bilateral_blur_preserves_dimensions() {
        let op = BilateralBlurOp::new(2, 5.0, 50.0).unwrap();
        let result = op.apply(test_image()).unwrap();
        assert_eq!(result.width(), 8);
        assert_eq!(result.height(), 8);
    }

    #[test]
    fn bilateral_blur_preserves_edges() {
        // Create an image with a sharp edge: left half black, right half white
        let edge_img = DynamicImage::ImageRgba8(image::RgbaImage::from_fn(16, 16, |x, _y| {
            if x < 8 {
                image::Rgba([0, 0, 0, 255])
            } else {
                image::Rgba([255, 255, 255, 255])
            }
        }));

        let op = BilateralBlurOp::new(3, 5.0, 10.0).unwrap();
        let result = op.apply(edge_img).unwrap();
        let rgba = result.to_rgba8();
        // Pixels far from the edge should remain close to original
        let dark = rgba.get_pixel(2, 8);
        let light = rgba.get_pixel(13, 8);
        assert!(dark[0] < 30, "dark side should stay dark, got {}", dark[0]);
        assert!(
            light[0] > 225,
            "light side should stay light, got {}",
            light[0]
        );
    }

    #[test]
    fn bilateral_blur_invalid_params() {
        assert!(BilateralBlurOp::new(0, 5.0, 50.0).is_err());
        assert!(BilateralBlurOp::new(2, -1.0, 50.0).is_err());
        assert!(BilateralBlurOp::new(2, 5.0, 0.0).is_err());
        assert!(BilateralBlurOp::new(2, 5.0, 256.0).is_err());
    }
}
