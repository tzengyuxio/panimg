//! ICC color profile handling.
//!
//! Provides profile embedding, extraction, color space conversion,
//! and profile information display using Little CMS 2.

use crate::error::{PanimgError, Result};
use image::DynamicImage;
use lcms2::{Intent, PixelFormat, Profile, Transform};
use serde::Serialize;
use std::path::Path;

/// Well-known color space profiles that can be used for conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    Srgb,
    AdobeRgb,
    DisplayP3,
}

impl std::str::FromStr for ColorSpace {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "srgb" => Ok(Self::Srgb),
            "adobe-rgb" | "adobergb" | "adobe_rgb" => Ok(Self::AdobeRgb),
            "display-p3" | "displayp3" | "display_p3" | "p3" => Ok(Self::DisplayP3),
            _ => Err(format!("unknown color space: '{s}'")),
        }
    }
}

impl ColorSpace {
    /// Display name of the color space.
    pub fn name(&self) -> &str {
        match self {
            Self::Srgb => "sRGB",
            Self::AdobeRgb => "Adobe RGB",
            Self::DisplayP3 => "Display P3",
        }
    }

    /// All supported color spaces.
    pub fn all() -> &'static [Self] {
        &[Self::Srgb, Self::AdobeRgb, Self::DisplayP3]
    }

    /// Create an lcms2 Profile for this color space.
    pub fn to_profile(&self) -> Result<Profile> {
        match self {
            Self::Srgb => Ok(Profile::new_srgb()),
            Self::AdobeRgb => {
                create_adobe_rgb_profile().map_err(|e| PanimgError::InvalidArgument {
                    message: format!("failed to create Adobe RGB profile: {e}"),
                    suggestion: "this is an internal error".into(),
                })
            }
            Self::DisplayP3 => {
                create_display_p3_profile().map_err(|e| PanimgError::InvalidArgument {
                    message: format!("failed to create Display P3 profile: {e}"),
                    suggestion: "this is an internal error".into(),
                })
            }
        }
    }
}

/// Information extracted from an ICC profile.
#[derive(Debug, Clone, Serialize)]
pub struct IccProfileInfo {
    pub description: String,
    pub color_space: String,
    pub pcs: String,
    pub version: String,
    pub device_class: String,
}

/// Extract ICC profile information from raw profile data.
pub fn profile_info_from_bytes(data: &[u8]) -> Result<IccProfileInfo> {
    let profile = Profile::new_icc(data).map_err(|e| PanimgError::InvalidArgument {
        message: format!("invalid ICC profile: {e}"),
        suggestion: "check that the ICC profile file is valid".into(),
    })?;

    Ok(extract_profile_info(&profile))
}

/// Extract human-readable info from an lcms2 Profile.
fn extract_profile_info(profile: &Profile) -> IccProfileInfo {
    let description = profile
        .info(lcms2::InfoType::Description, lcms2::Locale::none())
        .unwrap_or_else(|| "unknown".into());

    let color_space = format!("{:?}", profile.color_space());
    let pcs = format!("{:?}", profile.pcs());

    let version = format!("{}", profile.version());

    let device_class = format!("{:?}", profile.device_class());

    IccProfileInfo {
        description,
        color_space,
        pcs,
        version,
        device_class,
    }
}

/// Extract raw ICC profile bytes from a PNG file.
pub fn extract_icc_from_png(data: &[u8]) -> Option<Vec<u8>> {
    // PNG stores ICC profile in an iCCP chunk.
    let mut pos = 8; // Skip PNG signature
    while pos + 8 <= data.len() {
        let chunk_len =
            u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        let chunk_type = &data[pos + 4..pos + 8];

        // Validate chunk_len doesn't exceed remaining data
        let chunk_end = pos.checked_add(8)?.checked_add(chunk_len)?;
        if chunk_end > data.len() {
            return None;
        }

        if chunk_type == b"iCCP" {
            let chunk_data = &data[pos + 8..chunk_end];
            // iCCP format: profile name (null-terminated), compression method (1 byte), compressed data
            if let Some(null_pos) = chunk_data.iter().position(|&b| b == 0) {
                if null_pos + 2 <= chunk_data.len() {
                    let compressed = &chunk_data[null_pos + 2..];
                    if let Ok(decompressed) = miniz_decompress(compressed) {
                        return Some(decompressed);
                    }
                }
            }
            return None;
        }

        // Move to next chunk: length(4) + type(4) + data(chunk_len) + crc(4)
        pos = pos.checked_add(12 + chunk_len)?;
    }
    None
}

/// Extract raw ICC profile bytes from a JPEG file.
pub fn extract_icc_from_jpeg(data: &[u8]) -> Option<Vec<u8>> {
    // JPEG stores ICC profile in APP2 markers with "ICC_PROFILE\0" signature
    let mut segments: Vec<(u8, u8, Vec<u8>)> = Vec::new(); // (seq_no, total, data)
    let mut pos = 2; // Skip SOI marker

    while pos + 4 <= data.len() {
        if data[pos] != 0xFF {
            break;
        }
        let marker = data[pos + 1];

        // End of markers
        if marker == 0xDA {
            break;
        }

        // Markers without length
        if marker == 0xD8 || marker == 0xD9 || (0xD0..=0xD7).contains(&marker) {
            pos += 2;
            continue;
        }

        if pos + 4 > data.len() {
            break;
        }
        let seg_len = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;

        // Validate segment doesn't exceed remaining data
        let seg_end = match (pos + 2).checked_add(seg_len) {
            Some(end) if end <= data.len() => end,
            _ => break,
        };

        if marker == 0xE2 && seg_len > 16 {
            // APP2
            let seg_data = &data[pos + 4..seg_end];
            if seg_data.len() >= 14 && seg_data.starts_with(b"ICC_PROFILE\0") {
                let seq_no = seg_data[12];
                let total = seg_data[13];
                let profile_data = seg_data[14..].to_vec();
                segments.push((seq_no, total, profile_data));
            }
        }

        pos = seg_end;
    }

    if segments.is_empty() {
        return None;
    }

    segments.sort_by_key(|s| s.0);
    let mut result = Vec::new();
    for (_, _, data) in segments {
        result.extend_from_slice(&data);
    }
    Some(result)
}

/// Try to extract ICC profile from image file bytes based on format.
pub fn extract_icc_from_image(data: &[u8]) -> Option<Vec<u8>> {
    // Try PNG first
    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        return extract_icc_from_png(data);
    }
    // Try JPEG
    if data.starts_with(&[0xFF, 0xD8]) {
        return extract_icc_from_jpeg(data);
    }
    None
}

/// Convert image pixels from one ICC profile to another.
pub fn convert_image_profile(
    img: &DynamicImage,
    source_profile: &Profile,
    dest_profile: &Profile,
) -> Result<DynamicImage> {
    let make_transform_err = |e: lcms2::Error| PanimgError::InvalidArgument {
        message: format!("failed to create color transform: {e}"),
        suggestion: "check that the ICC profiles are compatible".into(),
    };

    match img {
        DynamicImage::ImageRgb8(rgb) => {
            let transform = Transform::new(
                source_profile,
                PixelFormat::RGB_8,
                dest_profile,
                PixelFormat::RGB_8,
                Intent::Perceptual,
            )
            .map_err(make_transform_err)?;

            let mut output = rgb.clone();
            let src_pixels: Vec<[u8; 3]> = rgb.pixels().map(|p| p.0).collect();
            transform.transform_pixels(&src_pixels, output.as_mut());
            Ok(DynamicImage::ImageRgb8(output))
        }
        DynamicImage::ImageRgba8(rgba) => {
            let transform = Transform::new(
                source_profile,
                PixelFormat::RGBA_8,
                dest_profile,
                PixelFormat::RGBA_8,
                Intent::Perceptual,
            )
            .map_err(make_transform_err)?;

            let mut output = rgba.clone();
            let src_pixels: Vec<[u8; 4]> = rgba.pixels().map(|p| p.0).collect();
            transform.transform_pixels(&src_pixels, output.as_mut());
            Ok(DynamicImage::ImageRgba8(output))
        }
        _ => {
            // Convert to RGBA8 first, then transform
            let rgba = img.to_rgba8();
            convert_image_profile(
                &DynamicImage::ImageRgba8(rgba),
                source_profile,
                dest_profile,
            )
        }
    }
}

/// Convert an image from the source profile to a named color space.
pub fn convert_to_color_space(
    img: &DynamicImage,
    source_data: Option<&[u8]>,
    target: ColorSpace,
) -> Result<DynamicImage> {
    let source_profile = if let Some(data) = source_data {
        Profile::new_icc(data).map_err(|e| PanimgError::InvalidArgument {
            message: format!("invalid source ICC profile: {e}"),
            suggestion: "the embedded ICC profile may be corrupted".into(),
        })?
    } else {
        // Assume sRGB if no profile is embedded
        Profile::new_srgb()
    };

    let dest_profile = target.to_profile()?;
    convert_image_profile(img, &source_profile, &dest_profile)
}

/// Load an ICC profile from a file path.
pub fn load_profile_from_file(path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            PanimgError::FileNotFound {
                path: path.to_path_buf(),
                suggestion: "check that the ICC profile path is correct".into(),
            }
        } else {
            PanimgError::IoError {
                message: e.to_string(),
                path: Some(path.to_path_buf()),
                suggestion: "check file permissions".into(),
            }
        }
    })
}

/// Create an Adobe RGB (1998) profile using lcms2 primaries.
fn create_adobe_rgb_profile() -> std::result::Result<Profile, lcms2::Error> {
    use lcms2::{CIExyY, CIExyYTRIPLE, ToneCurve};

    let d65 = CIExyY {
        x: 0.3127,
        y: 0.3290,
        Y: 1.0,
    };

    let primaries = CIExyYTRIPLE {
        Red: CIExyY {
            x: 0.6400,
            y: 0.3300,
            Y: 1.0,
        },
        Green: CIExyY {
            x: 0.2100,
            y: 0.7100,
            Y: 1.0,
        },
        Blue: CIExyY {
            x: 0.1500,
            y: 0.0600,
            Y: 1.0,
        },
    };

    // Adobe RGB uses gamma 2.2
    let gamma = ToneCurve::new(2.19921875);
    let curves = [&gamma, &gamma, &gamma];
    Profile::new_rgb(&d65, &primaries, &curves)
}

/// Create a Display P3 profile using lcms2 primaries.
fn create_display_p3_profile() -> std::result::Result<Profile, lcms2::Error> {
    use lcms2::{CIExyY, CIExyYTRIPLE, ToneCurve};

    let d65 = CIExyY {
        x: 0.3127,
        y: 0.3290,
        Y: 1.0,
    };

    let primaries = CIExyYTRIPLE {
        Red: CIExyY {
            x: 0.6800,
            y: 0.3200,
            Y: 1.0,
        },
        Green: CIExyY {
            x: 0.2650,
            y: 0.6900,
            Y: 1.0,
        },
        Blue: CIExyY {
            x: 0.1500,
            y: 0.0600,
            Y: 1.0,
        },
    };

    // Display P3 uses sRGB TRC (approximately 2.2 gamma with linear segment)
    // Use a simple gamma of 2.2 as approximation
    let gamma = ToneCurve::new(2.2);
    let curves = [&gamma, &gamma, &gamma];
    Profile::new_rgb(&d65, &primaries, &curves)
}

/// Simple zlib/deflate decompression for ICC data in PNG iCCP chunks.
fn miniz_decompress(data: &[u8]) -> std::result::Result<Vec<u8>, String> {
    // Use flate2 via the image crate's dependency, or implement minimal inflate.
    // Since the `image` crate pulls in flate2, we can use a simple approach.
    // Actually, let's use a manual zlib inflate with raw deflate.
    use std::io::Read;
    let mut decoder = flate2::read::ZlibDecoder::new(data);
    let mut result = Vec::new();
    decoder
        .read_to_end(&mut result)
        .map_err(|e| e.to_string())?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_color_space_srgb() {
        assert_eq!("srgb".parse::<ColorSpace>().ok(), Some(ColorSpace::Srgb));
        assert_eq!("sRGB".parse::<ColorSpace>().ok(), Some(ColorSpace::Srgb));
    }

    #[test]
    fn parse_color_space_adobe_rgb() {
        assert_eq!(
            "adobe-rgb".parse::<ColorSpace>().ok(),
            Some(ColorSpace::AdobeRgb)
        );
        assert_eq!(
            "adobergb".parse::<ColorSpace>().ok(),
            Some(ColorSpace::AdobeRgb)
        );
    }

    #[test]
    fn parse_color_space_display_p3() {
        assert_eq!(
            "display-p3".parse::<ColorSpace>().ok(),
            Some(ColorSpace::DisplayP3)
        );
        assert_eq!("p3".parse::<ColorSpace>().ok(), Some(ColorSpace::DisplayP3));
    }

    #[test]
    fn parse_color_space_invalid() {
        assert!("invalid".parse::<ColorSpace>().is_err());
    }

    #[test]
    fn create_profiles() {
        assert!(ColorSpace::Srgb.to_profile().is_ok());
        assert!(ColorSpace::AdobeRgb.to_profile().is_ok());
        assert!(ColorSpace::DisplayP3.to_profile().is_ok());
    }

    #[test]
    fn convert_srgb_to_adobe_rgb() {
        let img = DynamicImage::ImageRgb8(image::RgbImage::from_fn(2, 2, |_, _| {
            image::Rgb([128, 64, 32])
        }));

        let source = Profile::new_srgb();
        let dest = ColorSpace::AdobeRgb.to_profile().unwrap();
        let result = convert_image_profile(&img, &source, &dest);
        assert!(result.is_ok());

        let converted = result.unwrap();
        assert_eq!(converted.width(), 2);
        assert_eq!(converted.height(), 2);
    }

    #[test]
    fn convert_rgba_image() {
        let img = DynamicImage::ImageRgba8(image::RgbaImage::from_fn(2, 2, |_, _| {
            image::Rgba([128, 64, 32, 200])
        }));

        let source = Profile::new_srgb();
        let dest = ColorSpace::DisplayP3.to_profile().unwrap();
        let result = convert_image_profile(&img, &source, &dest);
        assert!(result.is_ok());
    }

    #[test]
    fn convert_to_color_space_without_source() {
        let img = DynamicImage::ImageRgb8(image::RgbImage::from_fn(4, 4, |_, _| {
            image::Rgb([200, 100, 50])
        }));

        // No source ICC data means assume sRGB
        let result = convert_to_color_space(&img, None, ColorSpace::AdobeRgb);
        assert!(result.is_ok());
    }

    #[test]
    fn profile_info_roundtrip() {
        let profile = Profile::new_srgb();
        let info = extract_profile_info(&profile);
        assert!(!info.description.is_empty());
    }

    #[test]
    fn extract_icc_from_non_icc_png() {
        let data = [
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, // IHDR length
            0x49, 0x48, 0x44, 0x52, // IHDR type
        ];
        assert!(extract_icc_from_png(&data).is_none());
    }

    #[test]
    fn extract_icc_from_non_icc_jpeg() {
        let data = [0xFF, 0xD8, 0xFF, 0xDA];
        assert!(extract_icc_from_jpeg(&data).is_none());
    }

    #[test]
    fn extract_icc_auto_detect_format() {
        // PNG without ICC
        let png_data = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert!(extract_icc_from_image(&png_data).is_none());

        // JPEG without ICC
        let jpeg_data = [0xFF, 0xD8, 0xFF, 0xDA];
        assert!(extract_icc_from_image(&jpeg_data).is_none());

        // Unknown format
        let unknown = [0x00, 0x01, 0x02, 0x03];
        assert!(extract_icc_from_image(&unknown).is_none());
    }
}
