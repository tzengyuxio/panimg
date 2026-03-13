use serde::Serialize;
use std::path::Path;

/// Supported image formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    Jpeg,
    Png,
    WebP,
    Avif,
    Tiff,
    Gif,
    Bmp,
    Qoi,
    Jxl,
    Svg,
    Pdf,
    Heic,
}

impl ImageFormat {
    /// Detect format from magic bytes at the start of file data.
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 4 {
            return None;
        }

        // JPEG: FF D8 FF
        if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Some(Self::Jpeg);
        }
        // PNG: 89 50 4E 47
        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            return Some(Self::Png);
        }
        // GIF: GIF87a or GIF89a
        if data.starts_with(b"GIF8") {
            return Some(Self::Gif);
        }
        // BMP: BM
        if data.starts_with(b"BM") {
            return Some(Self::Bmp);
        }
        // TIFF: II (little-endian) or MM (big-endian)
        if data.starts_with(&[0x49, 0x49, 0x2A, 0x00])
            || data.starts_with(&[0x4D, 0x4D, 0x00, 0x2A])
        {
            return Some(Self::Tiff);
        }
        // WebP: RIFF....WEBP
        if data.len() >= 12 && data.starts_with(b"RIFF") && &data[8..12] == b"WEBP" {
            return Some(Self::WebP);
        }
        // QOI: qoif
        if data.starts_with(b"qoif") {
            return Some(Self::Qoi);
        }
        // AVIF/HEIF/HEIC: ....ftyp (ISOBMFF container)
        if data.len() >= 12 && &data[4..8] == b"ftyp" {
            let brand = &data[8..12];
            if brand == b"avif" || brand == b"avis" {
                return Some(Self::Avif);
            }
            if brand == b"heic" || brand == b"heix" || brand == b"hevc" || brand == b"hevx" {
                return Some(Self::Heic);
            }
            // mif1 is generic HEIF — scan compatible brands to disambiguate
            if brand == b"mif1" {
                // Read ftyp box size from first 4 bytes (big-endian u32)
                let box_size = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
                let box_end = box_size.min(data.len());
                // Compatible brands start at offset 16, each 4 bytes
                let mut offset = 16;
                while offset + 4 <= box_end {
                    let compat = &data[offset..offset + 4];
                    if compat == b"avif" || compat == b"avis" {
                        return Some(Self::Avif);
                    }
                    if compat == b"heic"
                        || compat == b"heix"
                        || compat == b"hevc"
                        || compat == b"hevx"
                    {
                        return Some(Self::Heic);
                    }
                    offset += 4;
                }
                // Default: treat mif1 as AVIF (most common case)
                return Some(Self::Avif);
            }
        }
        // JPEG XL: FF 0A (codestream) or 00 00 00 0C 4A 58 4C 20 (container)
        if data.starts_with(&[0xFF, 0x0A]) {
            return Some(Self::Jxl);
        }
        if data.len() >= 8 && data.starts_with(&[0x00, 0x00, 0x00, 0x0C, 0x4A, 0x58, 0x4C, 0x20]) {
            return Some(Self::Jxl);
        }
        // SVG: starts with < and contains <svg (simplified heuristic)
        if data.starts_with(b"<") || data.starts_with(b"\xEF\xBB\xBF<") {
            let text = std::str::from_utf8(&data[..data.len().min(1024)]).unwrap_or("");
            if text.contains("<svg") {
                return Some(Self::Svg);
            }
        }
        // PDF: %PDF
        if data.starts_with(b"%PDF") {
            return Some(Self::Pdf);
        }

        None
    }

    /// Detect format from file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "png" => Some(Self::Png),
            "webp" => Some(Self::WebP),
            "avif" => Some(Self::Avif),
            "tif" | "tiff" => Some(Self::Tiff),
            "gif" => Some(Self::Gif),
            "bmp" => Some(Self::Bmp),
            "qoi" => Some(Self::Qoi),
            "jxl" => Some(Self::Jxl),
            "svg" => Some(Self::Svg),
            "pdf" => Some(Self::Pdf),
            "heic" | "heif" => Some(Self::Heic),
            _ => None,
        }
    }

    /// Detect format from a file path: try magic bytes first, then extension.
    pub fn from_path(path: &Path) -> Option<Self> {
        // Try magic bytes first
        if let Ok(data) = std::fs::read(path) {
            if let Some(fmt) = Self::from_bytes(&data) {
                return Some(fmt);
            }
        }
        // Fallback to extension
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(Self::from_extension)
    }

    /// Detect format from extension only (for output paths that don't exist yet).
    pub fn from_path_extension(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(Self::from_extension)
    }

    /// Get the canonical file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::WebP => "webp",
            Self::Avif => "avif",
            Self::Tiff => "tiff",
            Self::Gif => "gif",
            Self::Bmp => "bmp",
            Self::Qoi => "qoi",
            Self::Jxl => "jxl",
            Self::Svg => "svg",
            Self::Pdf => "pdf",
            Self::Heic => "heic",
        }
    }

    /// Get the MIME type for this format.
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::WebP => "image/webp",
            Self::Avif => "image/avif",
            Self::Tiff => "image/tiff",
            Self::Gif => "image/gif",
            Self::Bmp => "image/bmp",
            Self::Qoi => "image/qoi",
            Self::Jxl => "image/jxl",
            Self::Svg => "image/svg+xml",
            Self::Pdf => "application/pdf",
            Self::Heic => "image/heic",
        }
    }

    /// Convert to the `image` crate's format enum.
    pub fn to_image_format(&self) -> Option<image::ImageFormat> {
        match self {
            Self::Jpeg => Some(image::ImageFormat::Jpeg),
            Self::Png => Some(image::ImageFormat::Png),
            Self::WebP => Some(image::ImageFormat::WebP),
            Self::Avif => Some(image::ImageFormat::Avif),
            Self::Tiff => Some(image::ImageFormat::Tiff),
            Self::Gif => Some(image::ImageFormat::Gif),
            Self::Bmp => Some(image::ImageFormat::Bmp),
            Self::Qoi => Some(image::ImageFormat::Qoi),
            Self::Heic => None,
            _ => None,
        }
    }

    /// All supported formats.
    pub fn all() -> &'static [Self] {
        &[
            Self::Jpeg,
            Self::Png,
            Self::WebP,
            Self::Avif,
            Self::Tiff,
            Self::Gif,
            Self::Bmp,
            Self::Qoi,
            Self::Jxl,
            Self::Svg,
            Self::Pdf,
            Self::Heic,
        ]
    }

    /// Whether this format is available for encoding in the current build.
    pub fn can_encode(&self) -> bool {
        match self {
            Self::Jpeg
            | Self::Png
            | Self::WebP
            | Self::Bmp
            | Self::Gif
            | Self::Tiff
            | Self::Qoi => true,
            Self::Avif => cfg!(feature = "avif"),
            Self::Jxl | Self::Svg | Self::Pdf | Self::Heic => false,
        }
    }

    /// Whether this format is available for decoding in the current build.
    pub fn can_decode(&self) -> bool {
        match self {
            Self::Jpeg
            | Self::Png
            | Self::WebP
            | Self::Bmp
            | Self::Gif
            | Self::Tiff
            | Self::Qoi
            | Self::Avif => true,
            Self::Jxl => cfg!(feature = "jxl"),
            Self::Svg => cfg!(feature = "svg"),
            Self::Pdf => cfg!(feature = "pdf"),
            Self::Heic => cfg!(all(feature = "heic", target_vendor = "apple")),
        }
    }
}

impl std::fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Jpeg => write!(f, "JPEG"),
            Self::Png => write!(f, "PNG"),
            Self::WebP => write!(f, "WebP"),
            Self::Avif => write!(f, "AVIF"),
            Self::Tiff => write!(f, "TIFF"),
            Self::Gif => write!(f, "GIF"),
            Self::Bmp => write!(f, "BMP"),
            Self::Qoi => write!(f, "QOI"),
            Self::Jxl => write!(f, "JPEG XL"),
            Self::Svg => write!(f, "SVG"),
            Self::Pdf => write!(f, "PDF"),
            Self::Heic => write!(f, "HEIC"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_jpeg_magic() {
        let data = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        assert_eq!(ImageFormat::from_bytes(&data), Some(ImageFormat::Jpeg));
    }

    #[test]
    fn detect_png_magic() {
        let data = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(ImageFormat::from_bytes(&data), Some(ImageFormat::Png));
    }

    #[test]
    fn detect_webp_magic() {
        let mut data = vec![0u8; 12];
        data[..4].copy_from_slice(b"RIFF");
        data[8..12].copy_from_slice(b"WEBP");
        assert_eq!(ImageFormat::from_bytes(&data), Some(ImageFormat::WebP));
    }

    #[test]
    fn detect_gif_magic() {
        assert_eq!(
            ImageFormat::from_bytes(b"GIF89a\x01\x00"),
            Some(ImageFormat::Gif)
        );
    }

    #[test]
    fn detect_bmp_magic() {
        assert_eq!(
            ImageFormat::from_bytes(b"BM\x00\x00\x00\x00"),
            Some(ImageFormat::Bmp)
        );
    }

    #[test]
    fn detect_heic_magic() {
        // ftyp box with "heic" brand
        let mut data = vec![0u8; 12];
        data[4..8].copy_from_slice(b"ftyp");
        data[8..12].copy_from_slice(b"heic");
        assert_eq!(ImageFormat::from_bytes(&data), Some(ImageFormat::Heic));

        // ftyp box with "heix" brand
        data[8..12].copy_from_slice(b"heix");
        assert_eq!(ImageFormat::from_bytes(&data), Some(ImageFormat::Heic));

        // ftyp box with "hevc" brand
        data[8..12].copy_from_slice(b"hevc");
        assert_eq!(ImageFormat::from_bytes(&data), Some(ImageFormat::Heic));

        // ftyp box with "hevx" brand
        data[8..12].copy_from_slice(b"hevx");
        assert_eq!(ImageFormat::from_bytes(&data), Some(ImageFormat::Heic));
    }

    #[test]
    fn detect_heic_from_extension() {
        assert_eq!(ImageFormat::from_extension("heic"), Some(ImageFormat::Heic));
        assert_eq!(ImageFormat::from_extension("heif"), Some(ImageFormat::Heic));
        assert_eq!(ImageFormat::from_extension("HEIC"), Some(ImageFormat::Heic));
    }

    #[test]
    fn detect_from_extension() {
        assert_eq!(ImageFormat::from_extension("jpg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("JPEG"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("png"), Some(ImageFormat::Png));
        assert_eq!(ImageFormat::from_extension("webp"), Some(ImageFormat::WebP));
        assert_eq!(ImageFormat::from_extension("xyz"), None);
    }

    #[test]
    fn detect_pdf_magic() {
        assert_eq!(
            ImageFormat::from_bytes(b"%PDF-1.4 some content"),
            Some(ImageFormat::Pdf)
        );
    }

    #[test]
    fn detect_pdf_extension() {
        assert_eq!(ImageFormat::from_extension("pdf"), Some(ImageFormat::Pdf));
        assert_eq!(ImageFormat::from_extension("PDF"), Some(ImageFormat::Pdf));
    }

    #[test]
    fn pdf_can_encode_false() {
        assert!(!ImageFormat::Pdf.can_encode());
    }

    #[test]
    fn pdf_can_decode_depends_on_feature() {
        // This test verifies can_decode returns the correct value
        // based on whether the pdf feature is enabled.
        let can_decode = ImageFormat::Pdf.can_decode();
        if cfg!(feature = "pdf") {
            assert!(can_decode);
        } else {
            assert!(!can_decode);
        }
    }

    #[test]
    fn pdf_format_properties() {
        assert_eq!(ImageFormat::Pdf.extension(), "pdf");
        assert_eq!(ImageFormat::Pdf.mime_type(), "application/pdf");
        assert_eq!(ImageFormat::Pdf.to_image_format(), None);
        assert_eq!(ImageFormat::Pdf.to_string(), "PDF");
    }

    #[test]
    fn detect_mif1_with_avif_compat() {
        // mif1 with avif compatible brand → AVIF
        let mut data = vec![0u8; 24];
        // ftyp box size = 24
        data[0..4].copy_from_slice(&24u32.to_be_bytes());
        data[4..8].copy_from_slice(b"ftyp");
        data[8..12].copy_from_slice(b"mif1");
        // minor version (4 bytes)
        data[16..20].copy_from_slice(b"avif");
        assert_eq!(ImageFormat::from_bytes(&data), Some(ImageFormat::Avif));
    }

    #[test]
    fn detect_mif1_with_heic_compat() {
        // mif1 with heic compatible brand → HEIC
        let mut data = vec![0u8; 24];
        data[0..4].copy_from_slice(&24u32.to_be_bytes());
        data[4..8].copy_from_slice(b"ftyp");
        data[8..12].copy_from_slice(b"mif1");
        data[16..20].copy_from_slice(b"heic");
        assert_eq!(ImageFormat::from_bytes(&data), Some(ImageFormat::Heic));
    }

    #[test]
    fn detect_mif1_default_avif() {
        // mif1 with no recognizable compatible brand → defaults to AVIF
        let mut data = vec![0u8; 20];
        data[0..4].copy_from_slice(&20u32.to_be_bytes());
        data[4..8].copy_from_slice(b"ftyp");
        data[8..12].copy_from_slice(b"mif1");
        data[16..20].copy_from_slice(b"miaf");
        assert_eq!(ImageFormat::from_bytes(&data), Some(ImageFormat::Avif));
    }

    #[test]
    fn unknown_bytes_returns_none() {
        assert_eq!(ImageFormat::from_bytes(&[0x00, 0x01, 0x02, 0x03]), None);
    }
}
