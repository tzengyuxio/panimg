use crate::error::{PanimgError, Result};
use crate::format::ImageFormat;
use serde::Serialize;
use std::path::Path;

const INCHES_PER_CM: f64 = 2.54;
const CM_PER_METER: f64 = 100.0;

/// Unit for expressing image resolution/density.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ResolutionUnit {
    Dpi,
    Dpcm,
}

impl ResolutionUnit {
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "dpi" => Ok(Self::Dpi),
            "dpcm" => Ok(Self::Dpcm),
            _ => Err(PanimgError::InvalidArgument {
                message: format!("unknown resolution unit: '{s}'"),
                suggestion: "use one of: dpi, dpcm".into(),
            }),
        }
    }
}

impl std::fmt::Display for ResolutionUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dpi => write!(f, "dpi"),
            Self::Dpcm => write!(f, "dpcm"),
        }
    }
}

/// Image resolution, stored internally as DPI (dots per inch).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Resolution {
    pub x_dpi: f64,
    pub y_dpi: f64,
}

impl Resolution {
    /// Create a Resolution from a density value and unit.
    /// For uniform density (same X and Y).
    pub fn from_density(value: f64, unit: ResolutionUnit) -> Self {
        let dpi = match unit {
            ResolutionUnit::Dpi => value,
            ResolutionUnit::Dpcm => value * INCHES_PER_CM,
        };
        Self {
            x_dpi: dpi,
            y_dpi: dpi,
        }
    }

    /// Convert the internal DPI value to the requested unit.
    pub fn to_unit(&self, unit: ResolutionUnit) -> (f64, f64) {
        match unit {
            ResolutionUnit::Dpi => (self.x_dpi, self.y_dpi),
            ResolutionUnit::Dpcm => (self.x_dpi / INCHES_PER_CM, self.y_dpi / INCHES_PER_CM),
        }
    }
}

/// Read resolution from a file's EXIF metadata.
/// Returns `None` if the file has no resolution tags.
pub fn read_resolution(path: &Path) -> Option<Resolution> {
    let file = std::fs::File::open(path).ok()?;
    let mut bufreader = std::io::BufReader::new(file);
    let exif = exif::Reader::new()
        .read_from_container(&mut bufreader)
        .ok()?;

    // Read ResolutionUnit tag (default: 2 = inches per EXIF spec)
    let unit_value = exif
        .get_field(exif::Tag::ResolutionUnit, exif::In::PRIMARY)
        .and_then(|f| match &f.value {
            exif::Value::Short(v) => v.first().copied(),
            _ => None,
        })
        .unwrap_or(2); // 2 = inches, 3 = centimeters

    let x_res = exif
        .get_field(exif::Tag::XResolution, exif::In::PRIMARY)
        .and_then(rational_to_f64)?;
    let y_res = exif
        .get_field(exif::Tag::YResolution, exif::In::PRIMARY)
        .and_then(rational_to_f64)?;

    let (x_dpi, y_dpi) = match unit_value {
        3 => (x_res * INCHES_PER_CM, y_res * INCHES_PER_CM), // cm → inches
        _ => (x_res, y_res),                                 // assume inches
    };

    Some(Resolution { x_dpi, y_dpi })
}

fn rational_to_f64(field: &exif::Field) -> Option<f64> {
    match &field.value {
        exif::Value::Rational(v) => v
            .first()
            .filter(|r| r.denom != 0)
            .map(|r| r.num as f64 / r.denom as f64),
        _ => None,
    }
}

/// Inject resolution metadata into encoded image bytes.
/// Returns modified bytes for JPEG and PNG; returns original bytes with a
/// warning for unsupported formats.
pub fn inject_resolution(
    data: Vec<u8>,
    format: ImageFormat,
    resolution: &Resolution,
) -> Result<Vec<u8>> {
    match format {
        ImageFormat::Jpeg => inject_jpeg_resolution(&data, resolution),
        ImageFormat::Png => inject_png_resolution(&data, resolution),
        _ => {
            eprintln!(
                "warning: resolution metadata not supported for {} format, skipping",
                format
            );
            Ok(data)
        }
    }
}

/// Inject/replace JFIF APP0 density in a JPEG.
fn inject_jpeg_resolution(data: &[u8], resolution: &Resolution) -> Result<Vec<u8>> {
    use img_parts::jpeg::Jpeg;
    use img_parts::Bytes;

    let mut jpeg =
        Jpeg::from_bytes(Bytes::copy_from_slice(data)).map_err(|e| PanimgError::EncodeError {
            message: format!("failed to parse JPEG for resolution injection: {e}"),
            path: None,
            suggestion: "the encoded JPEG data may be invalid".into(),
        })?;

    // Build a JFIF APP0 segment with the target density.
    // JFIF APP0 format:
    //   5 bytes: "JFIF\0" identifier
    //   2 bytes: version (1.01)
    //   1 byte:  density unit (1=dpi, 2=dpcm)
    //   2 bytes: X density (big-endian u16)
    //   2 bytes: Y density (big-endian u16)
    //   1 byte:  thumbnail width (0)
    //   1 byte:  thumbnail height (0)

    // Use DPI if values fit in u16, otherwise use DPCM (clamped to u16 max)
    let (unit_byte, x_val, y_val) = if resolution.x_dpi <= 65535.0 && resolution.y_dpi <= 65535.0 {
        (
            1u8,
            resolution.x_dpi.round() as u16,
            resolution.y_dpi.round() as u16,
        )
    } else {
        let (x_dpcm, y_dpcm) = resolution.to_unit(ResolutionUnit::Dpcm);
        (
            2u8,
            (x_dpcm.round() as u32).min(65535) as u16,
            (y_dpcm.round() as u32).min(65535) as u16,
        )
    };

    let mut app0_data = Vec::with_capacity(14);
    app0_data.extend_from_slice(b"JFIF\0"); // identifier
    app0_data.extend_from_slice(&[1, 1]); // version 1.01
    app0_data.push(unit_byte); // density unit
    app0_data.extend_from_slice(&x_val.to_be_bytes()); // X density
    app0_data.extend_from_slice(&y_val.to_be_bytes()); // Y density
    app0_data.push(0); // thumbnail width
    app0_data.push(0); // thumbnail height

    let app0_segment = img_parts::jpeg::JpegSegment::new_with_contents(
        img_parts::jpeg::markers::APP0,
        app0_data.into(),
    );

    // Remove existing APP0 (JFIF) segments, then insert at position 0
    let segments = jpeg.segments_mut();
    segments.retain(|seg| {
        // Only remove JFIF APP0, not other APP0 segments
        !(seg.marker() == img_parts::jpeg::markers::APP0 && seg.contents().starts_with(b"JFIF\0"))
    });
    segments.insert(0, app0_segment);

    let mut output = Vec::new();
    jpeg.encoder()
        .write_to(&mut output)
        .map_err(|e| PanimgError::EncodeError {
            message: format!("failed to write JPEG with resolution: {e}"),
            path: None,
            suggestion: "internal error during JPEG assembly".into(),
        })?;

    Ok(output)
}

/// Inject/replace pHYs chunk in a PNG.
fn inject_png_resolution(data: &[u8], resolution: &Resolution) -> Result<Vec<u8>> {
    use img_parts::png::Png;
    use img_parts::Bytes;

    let mut png =
        Png::from_bytes(Bytes::copy_from_slice(data)).map_err(|e| PanimgError::EncodeError {
            message: format!("failed to parse PNG for resolution injection: {e}"),
            path: None,
            suggestion: "the encoded PNG data may be invalid".into(),
        })?;

    // pHYs chunk: pixels per unit, with unit=1 meaning "meter"
    // Convert DPI → pixels per meter: dpi * 100 / 2.54
    let x_ppm = (resolution.x_dpi * CM_PER_METER / INCHES_PER_CM).round() as u32;
    let y_ppm = (resolution.y_dpi * CM_PER_METER / INCHES_PER_CM).round() as u32;

    let mut phys_data = Vec::with_capacity(9);
    phys_data.extend_from_slice(&x_ppm.to_be_bytes());
    phys_data.extend_from_slice(&y_ppm.to_be_bytes());
    phys_data.push(1); // unit = meter

    let phys_chunk = img_parts::png::PngChunk::new(*b"pHYs", phys_data.into());

    // Remove existing pHYs chunks
    let chunks = png.chunks_mut();
    chunks.retain(|chunk| chunk.kind() != *b"pHYs");

    // Insert pHYs before IDAT (after IHDR and other metadata chunks)
    let idat_pos = chunks
        .iter()
        .position(|c| c.kind() == *b"IDAT")
        .unwrap_or(chunks.len());
    chunks.insert(idat_pos, phys_chunk);

    let mut output = Vec::new();
    png.encoder()
        .write_to(&mut output)
        .map_err(|e| PanimgError::EncodeError {
            message: format!("failed to write PNG with resolution: {e}"),
            path: None,
            suggestion: "internal error during PNG assembly".into(),
        })?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_density_dpi() {
        let res = Resolution::from_density(300.0, ResolutionUnit::Dpi);
        assert!((res.x_dpi - 300.0).abs() < f64::EPSILON);
        assert!((res.y_dpi - 300.0).abs() < f64::EPSILON);
    }

    #[test]
    fn from_density_dpcm() {
        let res = Resolution::from_density(100.0, ResolutionUnit::Dpcm);
        assert!((res.x_dpi - 254.0).abs() < f64::EPSILON);
    }

    #[test]
    fn to_unit_roundtrip() {
        let res = Resolution::from_density(300.0, ResolutionUnit::Dpi);
        let (x_dpcm, _) = res.to_unit(ResolutionUnit::Dpcm);
        // 300 / 2.54 ≈ 118.11
        assert!((x_dpcm - 118.11023622047244).abs() < 0.001);
    }

    #[test]
    fn parse_unit() {
        assert_eq!(ResolutionUnit::parse("dpi").unwrap(), ResolutionUnit::Dpi);
        assert_eq!(ResolutionUnit::parse("DPI").unwrap(), ResolutionUnit::Dpi);
        assert_eq!(ResolutionUnit::parse("dpcm").unwrap(), ResolutionUnit::Dpcm);
        assert!(ResolutionUnit::parse("ppi").is_err());
    }
}
