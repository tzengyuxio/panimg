use crate::error::{PanimgError, Result};
use crate::ops::{Operation, OperationDescription};
use crate::schema::{CommandSchema, ParamRange, ParamSchema, ParamType};
use image::DynamicImage;
use serde::Serialize;

/// How the image should fit into the target dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FitMode {
    /// Scale down to fit within bounds, preserving aspect ratio.
    Contain,
    /// Scale up to cover bounds, preserving aspect ratio (may crop).
    Cover,
    /// Stretch to exactly fill bounds (may distort).
    Fill,
    /// Like contain, but only scale down (never enlarge).
    Inside,
    /// Like cover, but only scale down (never enlarge).
    Outside,
}

impl FitMode {
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "contain" => Ok(Self::Contain),
            "cover" => Ok(Self::Cover),
            "fill" => Ok(Self::Fill),
            "inside" => Ok(Self::Inside),
            "outside" => Ok(Self::Outside),
            _ => Err(PanimgError::InvalidArgument {
                message: format!("unknown fit mode: '{s}'"),
                suggestion: "use one of: contain, cover, fill, inside, outside".into(),
            }),
        }
    }
}

/// Resize filter algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResizeFilter {
    Lanczos3,
    CatmullRom,
    Nearest,
    Linear,
}

impl ResizeFilter {
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "lanczos3" | "lanczos" => Ok(Self::Lanczos3),
            "catmull-rom" | "catmullrom" | "cubic" => Ok(Self::CatmullRom),
            "nearest" | "nn" => Ok(Self::Nearest),
            "linear" | "bilinear" => Ok(Self::Linear),
            _ => Err(PanimgError::InvalidArgument {
                message: format!("unknown filter: '{s}'"),
                suggestion: "use one of: lanczos3, catmull-rom, nearest, linear".into(),
            }),
        }
    }

    fn to_fir_type(self) -> fast_image_resize::ResizeAlg {
        use fast_image_resize::{FilterType, ResizeAlg};
        match self {
            Self::Lanczos3 => ResizeAlg::Convolution(FilterType::Lanczos3),
            Self::CatmullRom => ResizeAlg::Convolution(FilterType::CatmullRom),
            Self::Nearest => ResizeAlg::Nearest,
            Self::Linear => ResizeAlg::Convolution(FilterType::Bilinear),
        }
    }
}

/// Resize operation.
pub struct ResizeOp {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fit: FitMode,
    pub filter: ResizeFilter,
}

impl ResizeOp {
    pub fn new(
        width: Option<u32>,
        height: Option<u32>,
        fit: FitMode,
        filter: ResizeFilter,
    ) -> Result<Self> {
        if width.is_none() && height.is_none() {
            return Err(PanimgError::InvalidArgument {
                message: "at least one of --width or --height is required".into(),
                suggestion: "specify --width, --height, or both".into(),
            });
        }
        Ok(Self {
            width,
            height,
            fit,
            filter,
        })
    }

    /// Calculate target dimensions from source dimensions.
    pub fn calculate_dimensions(&self, src_w: u32, src_h: u32) -> (u32, u32) {
        let target_w = self.width.unwrap_or(0);
        let target_h = self.height.unwrap_or(0);

        match self.fit {
            FitMode::Fill => {
                let w = if target_w > 0 { target_w } else { src_w };
                let h = if target_h > 0 { target_h } else { src_h };
                (w, h)
            }
            FitMode::Contain | FitMode::Inside => {
                let (w, h) = fit_contain(src_w, src_h, target_w, target_h);
                if self.fit == FitMode::Inside {
                    // Never enlarge
                    (w.min(src_w), h.min(src_h))
                } else {
                    (w, h)
                }
            }
            FitMode::Cover | FitMode::Outside => {
                let (w, h) = fit_cover(src_w, src_h, target_w, target_h);
                if self.fit == FitMode::Outside {
                    (w.min(src_w), h.min(src_h))
                } else {
                    (w, h)
                }
            }
        }
    }
}

fn fit_contain(src_w: u32, src_h: u32, target_w: u32, target_h: u32) -> (u32, u32) {
    if target_w == 0 && target_h == 0 {
        return (src_w, src_h);
    }
    let aspect = src_w as f64 / src_h as f64;
    if target_w == 0 {
        return (((target_h as f64) * aspect).round() as u32, target_h);
    }
    if target_h == 0 {
        return (target_w, ((target_w as f64) / aspect).round() as u32);
    }
    let scale_w = target_w as f64 / src_w as f64;
    let scale_h = target_h as f64 / src_h as f64;
    let scale = scale_w.min(scale_h);
    (
        (src_w as f64 * scale).round() as u32,
        (src_h as f64 * scale).round() as u32,
    )
}

fn fit_cover(src_w: u32, src_h: u32, target_w: u32, target_h: u32) -> (u32, u32) {
    if target_w == 0 && target_h == 0 {
        return (src_w, src_h);
    }
    let aspect = src_w as f64 / src_h as f64;
    if target_w == 0 {
        return (((target_h as f64) * aspect).round() as u32, target_h);
    }
    if target_h == 0 {
        return (target_w, ((target_w as f64) / aspect).round() as u32);
    }
    let scale_w = target_w as f64 / src_w as f64;
    let scale_h = target_h as f64 / src_h as f64;
    let scale = scale_w.max(scale_h);
    (
        (src_w as f64 * scale).round() as u32,
        (src_h as f64 * scale).round() as u32,
    )
}

impl Operation<DynamicImage, PanimgError> for ResizeOp {
    fn name(&self) -> &str {
        "resize"
    }

    fn apply(&self, img: DynamicImage) -> Result<DynamicImage> {
        let src_w = img.width();
        let src_h = img.height();
        let (dst_w, dst_h) = self.calculate_dimensions(src_w, src_h);

        if dst_w == 0 || dst_h == 0 {
            return Err(PanimgError::ResizeError {
                message: format!("calculated dimensions are zero: {dst_w}x{dst_h}"),
                suggestion: "check width/height values".into(),
            });
        }

        if dst_w == src_w && dst_h == src_h {
            return Ok(img);
        }

        use fast_image_resize as fir;

        // Convert to RGBA8 for processing
        let rgba = img.to_rgba8();
        let src_image =
            fir::images::Image::from_vec_u8(src_w, src_h, rgba.into_raw(), fir::PixelType::U8x4)
                .map_err(|e| PanimgError::ResizeError {
                    message: e.to_string(),
                    suggestion: "source image data may be invalid".into(),
                })?;

        let mut dst_image = fir::images::Image::new(dst_w, dst_h, fir::PixelType::U8x4);

        let mut resizer = fir::Resizer::new();
        resizer
            .resize(
                &src_image,
                &mut dst_image,
                &fir::ResizeOptions::new().resize_alg(self.filter.to_fir_type()),
            )
            .map_err(|e| PanimgError::ResizeError {
                message: e.to_string(),
                suggestion: "resize operation failed".into(),
            })?;

        let result_buf = dst_image.into_vec();
        image::RgbaImage::from_raw(dst_w, dst_h, result_buf)
            .map(DynamicImage::ImageRgba8)
            .ok_or_else(|| PanimgError::ResizeError {
                message: "failed to create output image".into(),
                suggestion: "this is an internal error".into(),
            })
    }

    fn describe(&self) -> OperationDescription {
        OperationDescription {
            operation: "resize".into(),
            params: serde_json::json!({
                "width": self.width,
                "height": self.height,
                "fit": self.fit,
                "filter": self.filter,
            }),
            description: format!(
                "Resize to {}x{} using {:?} fit and {:?} filter",
                self.width.map(|w| w.to_string()).unwrap_or("auto".into()),
                self.height.map(|h| h.to_string()).unwrap_or("auto".into()),
                self.fit,
                self.filter,
            ),
        }
    }

    fn schema() -> CommandSchema {
        CommandSchema {
            command: "resize".into(),
            description: "Resize an image".into(),
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
                    required: false,
                    description: "Target width in pixels".into(),
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
                    required: false,
                    description: "Target height in pixels".into(),
                    default: None,
                    choices: None,
                    range: Some(ParamRange {
                        min: 1.0,
                        max: 65535.0,
                    }),
                },
                ParamSchema {
                    name: "fit".into(),
                    param_type: ParamType::String,
                    required: false,
                    description: "How the image should fit into target dimensions".into(),
                    default: Some(serde_json::json!("contain")),
                    choices: Some(vec![
                        "contain".into(),
                        "cover".into(),
                        "fill".into(),
                        "inside".into(),
                        "outside".into(),
                    ]),
                    range: None,
                },
                ParamSchema {
                    name: "filter".into(),
                    param_type: ParamType::String,
                    required: false,
                    description: "Resize filter algorithm".into(),
                    default: Some(serde_json::json!("lanczos3")),
                    choices: Some(vec![
                        "lanczos3".into(),
                        "catmull-rom".into(),
                        "nearest".into(),
                        "linear".into(),
                    ]),
                    range: None,
                },
                ParamSchema {
                    name: "quality".into(),
                    param_type: ParamType::Integer,
                    required: false,
                    description: "Output quality (1-100, for lossy formats)".into(),
                    default: None,
                    choices: None,
                    range: Some(ParamRange {
                        min: 1.0,
                        max: 100.0,
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
    fn contain_width_only() {
        let op = ResizeOp::new(Some(50), None, FitMode::Contain, ResizeFilter::Lanczos3).unwrap();
        let (w, h) = op.calculate_dimensions(200, 100);
        assert_eq!(w, 50);
        assert_eq!(h, 25);
    }

    #[test]
    fn contain_height_only() {
        let op = ResizeOp::new(None, Some(50), FitMode::Contain, ResizeFilter::Lanczos3).unwrap();
        let (w, h) = op.calculate_dimensions(200, 100);
        assert_eq!(w, 100);
        assert_eq!(h, 50);
    }

    #[test]
    fn contain_both() {
        let op = ResizeOp::new(
            Some(100),
            Some(100),
            FitMode::Contain,
            ResizeFilter::Lanczos3,
        )
        .unwrap();
        let (w, h) = op.calculate_dimensions(200, 100);
        assert_eq!(w, 100);
        assert_eq!(h, 50);
    }

    #[test]
    fn cover_both() {
        let op =
            ResizeOp::new(Some(100), Some(100), FitMode::Cover, ResizeFilter::Lanczos3).unwrap();
        let (w, h) = op.calculate_dimensions(200, 100);
        assert_eq!(w, 200);
        assert_eq!(h, 100);
    }

    #[test]
    fn fill_mode() {
        let op =
            ResizeOp::new(Some(100), Some(100), FitMode::Fill, ResizeFilter::Lanczos3).unwrap();
        let (w, h) = op.calculate_dimensions(200, 100);
        assert_eq!(w, 100);
        assert_eq!(h, 100);
    }

    #[test]
    fn inside_no_enlarge() {
        let op = ResizeOp::new(
            Some(400),
            Some(300),
            FitMode::Inside,
            ResizeFilter::Lanczos3,
        )
        .unwrap();
        let (w, h) = op.calculate_dimensions(200, 100);
        // Should not enlarge beyond original
        assert_eq!(w, 200);
        assert_eq!(h, 100);
    }

    #[test]
    fn requires_at_least_one_dimension() {
        let result = ResizeOp::new(None, None, FitMode::Contain, ResizeFilter::Lanczos3);
        assert!(result.is_err());
    }
}
