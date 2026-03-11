use crate::error::{PanimgError, Result};
use crate::format::ImageFormat;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;

/// Metadata extracted from an image file.
#[derive(Debug, Clone, Serialize)]
pub struct ImageInfo {
    pub path: String,
    pub format: ImageFormat,
    pub width: u32,
    pub height: u32,
    pub color_type: String,
    pub bit_depth: u8,
    pub file_size: u64,
    pub has_alpha: bool,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub exif: BTreeMap<String, String>,
}

impl ImageInfo {
    /// Extract metadata from an image file.
    pub fn from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(PanimgError::FileNotFound {
                path: path.to_path_buf(),
                suggestion: "check that the file path is correct".into(),
            });
        }

        let metadata = std::fs::metadata(path).map_err(|e| PanimgError::IoError {
            message: e.to_string(),
            path: Some(path.to_path_buf()),
            suggestion: "check file permissions".into(),
        })?;
        let file_size = metadata.len();

        let data = std::fs::read(path).map_err(|e| PanimgError::IoError {
            message: e.to_string(),
            path: Some(path.to_path_buf()),
            suggestion: "check file permissions".into(),
        })?;

        let format = ImageFormat::from_bytes(&data)
            .or_else(|| {
                path.extension()
                    .and_then(|e| e.to_str())
                    .and_then(ImageFormat::from_extension)
            })
            .ok_or_else(|| PanimgError::UnknownFormat {
                path: path.to_path_buf(),
                suggestion: "specify the format explicitly or use a recognized extension".into(),
            })?;

        // Decode image to get dimensions and color info
        let img = image::load_from_memory(&data).map_err(|e| PanimgError::DecodeError {
            message: e.to_string(),
            path: Some(path.to_path_buf()),
            suggestion: "the file may be corrupted or in an unsupported format".into(),
        })?;

        let (color_type_str, bit_depth, has_alpha) = describe_color_type(img.color());

        // Extract EXIF data
        let exif = extract_exif(path);

        Ok(ImageInfo {
            path: path.display().to_string(),
            format,
            width: img.width(),
            height: img.height(),
            color_type: color_type_str,
            bit_depth,
            file_size,
            has_alpha,
            exif,
        })
    }

    /// Filter to only the specified fields.
    pub fn to_filtered_json(&self, fields: &[String]) -> serde_json::Value {
        let full = serde_json::to_value(self).unwrap_or_default();
        if fields.is_empty() {
            return full;
        }
        let obj = full.as_object().unwrap();
        let filtered: serde_json::Map<String, serde_json::Value> = obj
            .iter()
            .filter(|(k, _)| fields.iter().any(|f| f == *k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        serde_json::Value::Object(filtered)
    }

    /// Format as human-readable text.
    pub fn to_human_string(&self) -> String {
        let mut lines = vec![
            format!("File:       {}", self.path),
            format!("Format:     {}", self.format),
            format!("Dimensions: {}x{}", self.width, self.height),
            format!("Color:      {}", self.color_type),
            format!("Bit Depth:  {}", self.bit_depth),
            format!("Alpha:      {}", if self.has_alpha { "yes" } else { "no" }),
            format!("File Size:  {}", format_file_size(self.file_size)),
        ];
        if !self.exif.is_empty() {
            lines.push("EXIF:".to_string());
            for (key, value) in &self.exif {
                lines.push(format!("  {}: {}", key, value));
            }
        }
        lines.join("\n")
    }
}

fn describe_color_type(color: image::ColorType) -> (String, u8, bool) {
    match color {
        image::ColorType::L8 => ("grayscale".into(), 8, false),
        image::ColorType::La8 => ("grayscale+alpha".into(), 8, true),
        image::ColorType::Rgb8 => ("rgb".into(), 8, false),
        image::ColorType::Rgba8 => ("rgba".into(), 8, true),
        image::ColorType::L16 => ("grayscale".into(), 16, false),
        image::ColorType::La16 => ("grayscale+alpha".into(), 16, true),
        image::ColorType::Rgb16 => ("rgb".into(), 16, false),
        image::ColorType::Rgba16 => ("rgba".into(), 16, true),
        image::ColorType::Rgb32F => ("rgb".into(), 32, false),
        image::ColorType::Rgba32F => ("rgba".into(), 32, true),
        _ => ("unknown".into(), 0, false),
    }
}

fn extract_exif(path: &Path) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return map,
    };
    let mut bufreader = std::io::BufReader::new(file);
    let exif = match exif::Reader::new().read_from_container(&mut bufreader) {
        Ok(e) => e,
        Err(_) => return map,
    };
    for field in exif.fields() {
        let tag_name = format!("{}", field.tag);
        let value = field.display_value().with_unit(&exif).to_string();
        map.insert(tag_name, value);
    }
    map
}

fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_file_size_bytes() {
        assert_eq!(format_file_size(500), "500 B");
    }

    #[test]
    fn format_file_size_kb() {
        assert_eq!(format_file_size(2048), "2.00 KB");
    }

    #[test]
    fn format_file_size_mb() {
        assert_eq!(format_file_size(1_500_000), "1.43 MB");
    }

    #[test]
    fn color_type_description() {
        let (name, bits, alpha) = describe_color_type(image::ColorType::Rgba8);
        assert_eq!(name, "rgba");
        assert_eq!(bits, 8);
        assert!(alpha);
    }

    #[test]
    fn filtered_json_empty_returns_all() {
        let info = ImageInfo {
            path: "test.png".into(),
            format: crate::format::ImageFormat::Png,
            width: 100,
            height: 200,
            color_type: "rgba".into(),
            bit_depth: 8,
            file_size: 1024,
            has_alpha: true,
            exif: BTreeMap::new(),
        };
        let json = info.to_filtered_json(&[]);
        assert!(json.get("width").is_some());
        assert!(json.get("height").is_some());
    }

    #[test]
    fn filtered_json_specific_fields() {
        let info = ImageInfo {
            path: "test.png".into(),
            format: crate::format::ImageFormat::Png,
            width: 100,
            height: 200,
            color_type: "rgba".into(),
            bit_depth: 8,
            file_size: 1024,
            has_alpha: true,
            exif: BTreeMap::new(),
        };
        let json = info.to_filtered_json(&["width".into(), "height".into()]);
        let obj = json.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert_eq!(obj["width"], 100);
        assert_eq!(obj["height"], 200);
    }
}
