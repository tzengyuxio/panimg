use crate::error::{PanimgError, Result};
use crate::format::ImageFormat;
use image::DynamicImage;
use std::path::Path;

/// Options for encoding an image.
#[derive(Debug, Clone)]
pub struct EncodeOptions {
    pub format: ImageFormat,
    pub quality: Option<u8>,
    pub strip_metadata: bool,
}

impl Default for EncodeOptions {
    fn default() -> Self {
        Self {
            format: ImageFormat::Png,
            quality: None,
            strip_metadata: false,
        }
    }
}

/// Registry that handles decoding and encoding images.
pub struct CodecRegistry;

impl CodecRegistry {
    /// Decode an image from a file path.
    pub fn decode(path: &Path) -> Result<DynamicImage> {
        if !path.exists() {
            return Err(PanimgError::FileNotFound {
                path: path.to_path_buf(),
                suggestion: "check that the file path is correct".into(),
            });
        }

        let data = std::fs::read(path).map_err(|e| PanimgError::IoError {
            message: e.to_string(),
            path: Some(path.to_path_buf()),
            suggestion: "check file permissions".into(),
        })?;

        let format = ImageFormat::from_bytes(&data)
            .or_else(|| ImageFormat::from_path_extension(path))
            .ok_or_else(|| PanimgError::UnknownFormat {
                path: path.to_path_buf(),
                suggestion: "specify the format explicitly or use a recognized extension".into(),
            })?;

        Self::decode_bytes(&data, format, Some(path))
    }

    /// Decode an image from bytes with a known format.
    fn decode_bytes(data: &[u8], format: ImageFormat, path: Option<&Path>) -> Result<DynamicImage> {
        match format {
            ImageFormat::Jpeg
            | ImageFormat::Png
            | ImageFormat::WebP
            | ImageFormat::Gif
            | ImageFormat::Bmp
            | ImageFormat::Tiff
            | ImageFormat::Qoi
            | ImageFormat::Avif => {
                let img_fmt = format.to_image_format().unwrap();
                image::load_from_memory_with_format(data, img_fmt).map_err(|e| {
                    PanimgError::DecodeError {
                        message: e.to_string(),
                        path: path.map(|p| p.to_path_buf()),
                        suggestion: "the file may be corrupted".into(),
                    }
                })
            }
            #[cfg(feature = "svg")]
            ImageFormat::Svg => decode_svg(data, path),
            #[cfg(feature = "jxl")]
            ImageFormat::Jxl => decode_jxl(data, path),
            _ => Err(PanimgError::UnsupportedFormat {
                format: format.to_string(),
                suggestion: format!(
                    "enable the '{}' feature to support this format",
                    format.extension()
                ),
            }),
        }
    }

    /// Encode an image and write to a file path.
    pub fn encode(img: &DynamicImage, path: &Path, options: &EncodeOptions) -> Result<()> {
        if !options.format.can_encode() {
            return Err(PanimgError::UnsupportedFormat {
                format: options.format.to_string(),
                suggestion: "this format is not supported for encoding".into(),
            });
        }

        let img_fmt =
            options
                .format
                .to_image_format()
                .ok_or_else(|| PanimgError::UnsupportedFormat {
                    format: options.format.to_string(),
                    suggestion: "this format is not supported for encoding".into(),
                })?;

        // For JPEG, set quality
        if options.format == ImageFormat::Jpeg {
            let quality = options.quality.unwrap_or(85);
            let file = std::fs::File::create(path).map_err(|e| PanimgError::IoError {
                message: e.to_string(),
                path: Some(path.to_path_buf()),
                suggestion: "check output directory exists and permissions".into(),
            })?;
            let mut writer = std::io::BufWriter::new(file);
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, quality);
            img.write_with_encoder(encoder)
                .map_err(|e| PanimgError::EncodeError {
                    message: e.to_string(),
                    path: Some(path.to_path_buf()),
                    suggestion: "check that the image data is valid".into(),
                })?;
            return Ok(());
        }

        // Default: use image crate's save method
        img.save_with_format(path, img_fmt)
            .map_err(|e| PanimgError::EncodeError {
                message: e.to_string(),
                path: Some(path.to_path_buf()),
                suggestion: "check output directory exists and permissions".into(),
            })
    }
}

#[cfg(feature = "svg")]
fn decode_svg(data: &[u8], path: Option<&Path>) -> Result<DynamicImage> {
    let tree =
        resvg::usvg::Tree::from_data(data, &resvg::usvg::Options::default()).map_err(|e| {
            PanimgError::DecodeError {
                message: e.to_string(),
                path: path.map(|p| p.to_path_buf()),
                suggestion: "check that the SVG is well-formed".into(),
            }
        })?;
    let size = tree.size();
    let width = size.width() as u32;
    let height = size.height() as u32;
    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(width, height).ok_or_else(|| PanimgError::DecodeError {
            message: "failed to create pixmap".into(),
            path: path.map(|p| p.to_path_buf()),
            suggestion: "SVG dimensions may be invalid".into(),
        })?;
    resvg::render(
        &tree,
        resvg::usvg::Transform::default(),
        &mut pixmap.as_mut(),
    );
    let rgba_data = pixmap.data().to_vec();
    image::RgbaImage::from_raw(width, height, rgba_data)
        .map(DynamicImage::ImageRgba8)
        .ok_or_else(|| PanimgError::DecodeError {
            message: "failed to create image from SVG render".into(),
            path: path.map(|p| p.to_path_buf()),
            suggestion: "SVG dimensions may be invalid".into(),
        })
}

#[cfg(feature = "jxl")]
fn decode_jxl(data: &[u8], path: Option<&Path>) -> Result<DynamicImage> {
    use jxl_oxide::JxlImage;
    let image = JxlImage::builder()
        .read(std::io::Cursor::new(data))
        .map_err(|e| PanimgError::DecodeError {
            message: e.to_string(),
            path: path.map(|p| p.to_path_buf()),
            suggestion: "check that the JPEG XL file is valid".into(),
        })?;
    let render = image
        .render_frame(0)
        .map_err(|e| PanimgError::DecodeError {
            message: e.to_string(),
            path: path.map(|p| p.to_path_buf()),
            suggestion: "failed to render JPEG XL frame".into(),
        })?;
    let fb = render.image_all_channels();
    let width = fb.width() as u32;
    let height = fb.height() as u32;
    let buf: Vec<u8> = fb
        .buf()
        .iter()
        .map(|&f| (f.clamp(0.0, 1.0) * 255.0) as u8)
        .collect();
    let channels = fb.channels();
    match channels {
        3 => image::RgbImage::from_raw(width, height, buf)
            .map(DynamicImage::ImageRgb8)
            .ok_or_else(|| PanimgError::DecodeError {
                message: "failed to create image from JXL data".into(),
                path: path.map(|p| p.to_path_buf()),
                suggestion: "JXL data may be invalid".into(),
            }),
        4 => image::RgbaImage::from_raw(width, height, buf)
            .map(DynamicImage::ImageRgba8)
            .ok_or_else(|| PanimgError::DecodeError {
                message: "failed to create image from JXL data".into(),
                path: path.map(|p| p.to_path_buf()),
                suggestion: "JXL data may be invalid".into(),
            }),
        _ => Err(PanimgError::DecodeError {
            message: format!("unsupported channel count: {channels}"),
            path: path.map(|p| p.to_path_buf()),
            suggestion: "only RGB and RGBA JPEG XL images are supported".into(),
        }),
    }
}
