use crate::codec::{CodecRegistry, EncodeOptions};
use crate::error::{PanimgError, Result};
use crate::format::ImageFormat;
use serde::Serialize;
use std::path::Path;

/// Options for smart image compression.
#[derive(Debug, Clone)]
pub struct CompressOptions {
    pub quality: Option<u8>,
    pub max_colors: u16,
    pub lossless: bool,
    pub strip_metadata: bool,
}

impl Default for CompressOptions {
    fn default() -> Self {
        Self {
            quality: None,
            max_colors: 256,
            lossless: false,
            strip_metadata: false,
        }
    }
}

/// Result of a compression operation.
#[derive(Debug, Serialize)]
pub struct CompressResult {
    pub format: String,
    pub input_size: u64,
    pub output_size: u64,
    pub savings_percent: f64,
}

/// Compress an image file using format-appropriate strategies.
///
/// - **PNG**: lossy quantization (imagequant) + lossless optimization (oxipng)
/// - **JPEG**: quality-controlled re-encoding
/// - **WebP**: quality-controlled encoding
/// - **AVIF**: quality-controlled encoding (requires `avif` feature)
pub fn compress(input: &Path, output: &Path, options: &CompressOptions) -> Result<CompressResult> {
    let data = std::fs::read(input).map_err(|e| PanimgError::IoError {
        message: e.to_string(),
        path: Some(input.to_path_buf()),
        suggestion: "check file permissions".into(),
    })?;
    let input_size = data.len() as u64;

    let format = ImageFormat::from_bytes(&data)
        .or_else(|| ImageFormat::from_path_extension(input))
        .ok_or_else(|| PanimgError::UnknownFormat {
            path: input.to_path_buf(),
            suggestion: "the input file format could not be detected".into(),
        })?;

    match format {
        #[cfg(feature = "tiny")]
        ImageFormat::Png => compress_png(&data, output, options)?,
        #[cfg(not(feature = "tiny"))]
        ImageFormat::Png => {
            return Err(PanimgError::UnsupportedFormat {
                format: "PNG compression".into(),
                suggestion: "enable the 'tiny' feature for PNG quantization support".into(),
            });
        }
        ImageFormat::Jpeg => compress_via_codec(&data, output, options, ImageFormat::Jpeg, 75)?,
        ImageFormat::WebP => compress_via_codec(&data, output, options, ImageFormat::WebP, 75)?,
        #[cfg(feature = "avif")]
        ImageFormat::Avif => compress_via_codec(&data, output, options, ImageFormat::Avif, 68)?,
        _ => {
            return Err(PanimgError::UnsupportedFormat {
                format: format.to_string(),
                suggestion: "tiny supports PNG, JPEG, WebP, and AVIF formats".into(),
            });
        }
    }

    let output_size =
        std::fs::metadata(output)
            .map(|m| m.len())
            .map_err(|e| PanimgError::IoError {
                message: e.to_string(),
                path: Some(output.to_path_buf()),
                suggestion: "check output file".into(),
            })?;

    let savings_percent = if input_size > 0 {
        (1.0 - (output_size as f64 / input_size as f64)) * 100.0
    } else {
        0.0
    };

    Ok(CompressResult {
        format: format.to_string(),
        input_size,
        output_size,
        savings_percent,
    })
}

#[cfg(feature = "tiny")]
fn compress_png(data: &[u8], output: &Path, options: &CompressOptions) -> Result<()> {
    if options.lossless {
        let optimized = optimize_png_lossless(data, options.strip_metadata)?;
        return write_output(output, &optimized);
    }

    // Lossy: decode → imagequant quantize → png encode indexed → oxipng optimize
    let img = image::load_from_memory_with_format(data, image::ImageFormat::Png).map_err(|e| {
        PanimgError::DecodeError {
            message: e.to_string(),
            path: None,
            suggestion: "the file may be corrupted".into(),
        }
    })?;
    let rgba = img.to_rgba8();
    let width = rgba.width() as usize;
    let height = rgba.height() as usize;

    // Quantize with imagequant — RGBA memory layout is compatible, but we use
    // the safe borrowed API which requires imagequant::RGBA slice
    let mut liq = imagequant::new();
    liq.set_max_colors(options.max_colors as u32)
        .map_err(|e| PanimgError::EncodeError {
            message: format!("imagequant config error: {e}"),
            path: None,
            suggestion: "max-colors must be 2-256".into(),
        })?;

    let pixels: Vec<imagequant::RGBA> = rgba
        .pixels()
        .map(|p| imagequant::RGBA::new(p[0], p[1], p[2], p[3]))
        .collect();

    let mut img_liq = liq
        .new_image_borrowed(&pixels, width, height, 0.0)
        .map_err(|e| PanimgError::EncodeError {
            message: format!("imagequant image error: {e}"),
            path: None,
            suggestion: "image data may be invalid".into(),
        })?;

    let mut result = liq
        .quantize(&mut img_liq)
        .map_err(|e| PanimgError::EncodeError {
            message: format!("imagequant quantization failed: {e}"),
            path: None,
            suggestion: "try increasing --max-colors".into(),
        })?;

    let (palette, indexed_pixels) =
        result
            .remapped(&mut img_liq)
            .map_err(|e| PanimgError::EncodeError {
                message: format!("imagequant remapping failed: {e}"),
                path: None,
                suggestion: "image data may be invalid".into(),
            })?;

    // Encode as indexed PNG using the png crate
    let mut png_data = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png_data, width as u32, height as u32);
        encoder.set_color(png::ColorType::Indexed);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::default());

        // Build PLTE and tRNS chunks
        let mut plte = Vec::with_capacity(palette.len() * 3);
        let mut trns = Vec::with_capacity(palette.len());
        let mut has_transparency = false;

        for color in &palette {
            plte.push(color.r);
            plte.push(color.g);
            plte.push(color.b);
            trns.push(color.a);
            if color.a < 255 {
                has_transparency = true;
            }
        }

        encoder.set_palette(plte);
        if has_transparency {
            encoder.set_trns(trns);
        }

        let mut writer = encoder
            .write_header()
            .map_err(|e| PanimgError::EncodeError {
                message: format!("PNG header write failed: {e}"),
                path: None,
                suggestion: "image data may be invalid".into(),
            })?;

        writer
            .write_image_data(&indexed_pixels)
            .map_err(|e| PanimgError::EncodeError {
                message: format!("PNG data write failed: {e}"),
                path: None,
                suggestion: "image data may be invalid".into(),
            })?;
    }

    // Further optimize with oxipng
    let optimized = optimize_png_lossless(&png_data, options.strip_metadata)?;
    write_output(output, &optimized)
}

#[cfg(feature = "tiny")]
fn optimize_png_lossless(png_data: &[u8], strip_metadata: bool) -> Result<Vec<u8>> {
    let mut opts = oxipng::Options::default();
    if strip_metadata {
        opts.strip = oxipng::StripChunks::Safe;
    }
    opts.optimize_alpha = true;

    oxipng::optimize_from_memory(png_data, &opts).map_err(|e| PanimgError::EncodeError {
        message: format!("oxipng optimization failed: {e}"),
        path: None,
        suggestion: "the PNG data may be invalid".into(),
    })
}

/// Compress via decode → re-encode with quality. Used for JPEG, WebP, AVIF.
fn compress_via_codec(
    data: &[u8],
    output: &Path,
    options: &CompressOptions,
    format: ImageFormat,
    default_quality: u8,
) -> Result<()> {
    let img_fmt = format
        .to_image_format()
        .ok_or_else(|| PanimgError::UnsupportedFormat {
            format: format.to_string(),
            suggestion: "this format is not supported for encoding".into(),
        })?;
    let img = image::load_from_memory_with_format(data, img_fmt).map_err(|e| {
        PanimgError::DecodeError {
            message: e.to_string(),
            path: None,
            suggestion: "the file may be corrupted".into(),
        }
    })?;
    let quality = options.quality.unwrap_or(default_quality);
    let encode_options = EncodeOptions {
        format,
        quality: Some(quality),
        strip_metadata: options.strip_metadata,
    };
    CodecRegistry::encode(&img, output, &encode_options)
}

#[cfg(feature = "tiny")]
fn write_output(path: &Path, data: &[u8]) -> Result<()> {
    std::fs::write(path, data).map_err(|e| PanimgError::IoError {
        message: e.to_string(),
        path: Some(path.to_path_buf()),
        suggestion: "check output directory exists and permissions".into(),
    })
}
