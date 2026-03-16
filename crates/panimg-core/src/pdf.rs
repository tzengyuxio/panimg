use crate::error::{PanimgError, Result};
use image::DynamicImage;
use serde::Serialize;
use std::collections::BTreeSet;

/// A set of 0-based page indices, parsed from a 1-based user specification.
#[derive(Debug, Clone)]
pub struct PageRange(BTreeSet<usize>);

impl PageRange {
    /// Parse a page specification string (1-based) into a set of 0-based indices.
    ///
    /// Accepted formats: `"1"`, `"1-4"`, `"1,3,5"`, `"1-3,7,10-12"`.
    pub fn parse(spec: &str) -> Result<Self> {
        let spec = spec.trim();
        if spec.is_empty() {
            return Err(PanimgError::InvalidArgument {
                message: "empty page range specification".into(),
                suggestion: "use a format like '1', '1-3', or '1,3,5'".into(),
            });
        }

        let mut pages = BTreeSet::new();

        for part in spec.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            if let Some((start_str, end_str)) = part.split_once('-') {
                let start: usize =
                    start_str
                        .trim()
                        .parse()
                        .map_err(|_| PanimgError::InvalidArgument {
                            message: format!("invalid page number: '{}'", start_str.trim()),
                            suggestion: "page numbers must be positive integers".into(),
                        })?;
                let end: usize =
                    end_str
                        .trim()
                        .parse()
                        .map_err(|_| PanimgError::InvalidArgument {
                            message: format!("invalid page number: '{}'", end_str.trim()),
                            suggestion: "page numbers must be positive integers".into(),
                        })?;

                if start == 0 || end == 0 {
                    return Err(PanimgError::InvalidArgument {
                        message: "page numbers are 1-based, 0 is not valid".into(),
                        suggestion: "use page numbers starting from 1".into(),
                    });
                }
                if start > end {
                    return Err(PanimgError::InvalidArgument {
                        message: format!("invalid page range: {start}-{end} (start > end)"),
                        suggestion: "use ascending ranges like '3-5', not '5-3'".into(),
                    });
                }

                for p in start..=end {
                    pages.insert(p - 1); // Convert to 0-based
                }
            } else {
                let page: usize = part.parse().map_err(|_| PanimgError::InvalidArgument {
                    message: format!("invalid page number: '{part}'"),
                    suggestion: "page numbers must be positive integers".into(),
                })?;
                if page == 0 {
                    return Err(PanimgError::InvalidArgument {
                        message: "page numbers are 1-based, 0 is not valid".into(),
                        suggestion: "use page numbers starting from 1".into(),
                    });
                }
                pages.insert(page - 1); // Convert to 0-based
            }
        }

        if pages.is_empty() {
            return Err(PanimgError::InvalidArgument {
                message: "no valid pages in specification".into(),
                suggestion: "use a format like '1', '1-3', or '1,3,5'".into(),
            });
        }

        Ok(Self(pages))
    }

    /// Create a range covering all pages (0 to total-1).
    pub fn all(total: usize) -> Self {
        Self((0..total).collect())
    }

    /// Check if the range contains a 0-based page index.
    pub fn contains(&self, page: usize) -> bool {
        self.0.contains(&page)
    }

    /// Iterate over contained 0-based page indices.
    pub fn iter(&self) -> impl Iterator<Item = &usize> {
        self.0.iter()
    }
}

/// PDF metadata.
#[derive(Debug, Clone, Serialize)]
pub struct PdfInfo {
    pub page_count: usize,
}

/// Information about a single rendered page.
pub struct PageInfo {
    /// 0-based page index.
    pub index: usize,
    pub width: u32,
    pub height: u32,
}

/// An opaque wrapper around a parsed PDF document.
/// Hides the `hayro` crate's internal types from the public API.
pub struct PdfDocument(hayro::hayro_syntax::Pdf);

impl PdfDocument {
    /// Parse PDF data from a byte slice. The data is copied internally.
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        Self::from_vec(data.to_vec())
    }

    /// Parse PDF data from an owned buffer, avoiding an extra copy.
    pub fn from_vec(data: Vec<u8>) -> Result<Self> {
        let pdf_data: std::sync::Arc<dyn AsRef<[u8]> + Send + Sync> = std::sync::Arc::new(data);
        let pdf =
            hayro::hayro_syntax::Pdf::new(pdf_data).map_err(|e| PanimgError::DecodeError {
                message: format!("{e:?}"),
                path: None,
                suggestion: "check that the PDF file is valid and not encrypted".into(),
            })?;
        Ok(Self(pdf))
    }

    /// Return the total number of pages.
    pub fn page_count(&self) -> usize {
        self.0.pages().len()
    }

    /// Get PDF metadata.
    pub fn info(&self) -> PdfInfo {
        PdfInfo {
            page_count: self.page_count(),
        }
    }

    /// Render a single page (0-based index) at the given DPI.
    pub fn render_page(&self, page_index: usize, dpi: f32) -> Result<DynamicImage> {
        let pages = self.0.pages();
        if page_index >= pages.len() {
            return Err(PanimgError::InvalidArgument {
                message: format!(
                    "page {} out of range (PDF has {} pages)",
                    page_index + 1,
                    pages.len()
                ),
                suggestion: format!("use a page number between 1 and {}", pages.len()),
            });
        }

        render_page_inner(&pages[page_index], dpi)
    }

    /// Process each page in the given range through a callback, rendering one page
    /// at a time to avoid holding all decoded images in memory simultaneously.
    ///
    /// The callback receives `(PageInfo, DynamicImage)` and returns `Ok(true)` to
    /// continue or `Ok(false)` to stop early.
    ///
    /// Returns the total number of pages in the PDF.
    pub fn for_each_page<F>(&self, range: &PageRange, dpi: f32, mut f: F) -> Result<usize>
    where
        F: FnMut(PageInfo, DynamicImage) -> Result<bool>,
    {
        let pages = self.0.pages();
        let total = pages.len();

        for &page_index in range.iter() {
            if page_index >= total {
                continue;
            }

            // Skip bounds check — we already validated above
            let img = render_page_inner(&pages[page_index], dpi)?;
            let info = PageInfo {
                index: page_index,
                width: img.width(),
                height: img.height(),
            };

            if !f(info, img)? {
                break;
            }
        }

        Ok(total)
    }
}

/// Internal render helper that operates on a single hayro page reference.
fn render_page_inner(page: &hayro::hayro_syntax::page::Page<'_>, dpi: f32) -> Result<DynamicImage> {
    let scale = dpi / 72.0;
    let interpreter_settings = hayro::hayro_interpret::InterpreterSettings::default();
    let render_settings = hayro::RenderSettings {
        x_scale: scale,
        y_scale: scale,
        bg_color: hayro::vello_cpu::color::palette::css::WHITE,
        ..Default::default()
    };

    let pixmap = hayro::render(page, &interpreter_settings, &render_settings);
    let width = pixmap.width() as u32;
    let height = pixmap.height() as u32;
    let unpremultiplied = pixmap.take_unpremultiplied();
    let rgba_data: Vec<u8> = unpremultiplied
        .into_iter()
        .flat_map(|p| [p.r, p.g, p.b, p.a])
        .collect();

    image::RgbaImage::from_raw(width, height, rgba_data)
        .map(DynamicImage::ImageRgba8)
        .ok_or_else(|| PanimgError::DecodeError {
            message: "failed to create image from PDF render".into(),
            path: None,
            suggestion: "PDF page dimensions may be invalid".into(),
        })
}

// --- Convenience free functions (delegate to PdfDocument) ---

/// Get PDF metadata (page count) from raw bytes.
pub fn get_pdf_info(data: &[u8]) -> Result<PdfInfo> {
    let doc = PdfDocument::from_bytes(data)?;
    Ok(doc.info())
}

/// Process each page through a callback. Convenience wrapper around `PdfDocument`.
pub fn for_each_page<F>(data: &[u8], range: &PageRange, dpi: f32, f: F) -> Result<usize>
where
    F: FnMut(PageInfo, DynamicImage) -> Result<bool>,
{
    let doc = PdfDocument::from_bytes(data)?;
    doc.for_each_page(range, dpi, f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_page() {
        let range = PageRange::parse("1").unwrap();
        assert!(range.contains(0));
        assert!(!range.contains(1));
    }

    #[test]
    fn parse_range() {
        let range = PageRange::parse("1-3").unwrap();
        assert!(range.contains(0));
        assert!(range.contains(1));
        assert!(range.contains(2));
        assert!(!range.contains(3));
    }

    #[test]
    fn parse_comma_separated() {
        let range = PageRange::parse("1,3,5").unwrap();
        assert!(range.contains(0));
        assert!(!range.contains(1));
        assert!(range.contains(2));
        assert!(!range.contains(3));
        assert!(range.contains(4));
    }

    #[test]
    fn parse_mixed() {
        let range = PageRange::parse("1-3,7").unwrap();
        assert!(range.contains(0));
        assert!(range.contains(1));
        assert!(range.contains(2));
        assert!(!range.contains(3));
        assert!(range.contains(6));
    }

    #[test]
    fn parse_zero_is_error() {
        assert!(PageRange::parse("0").is_err());
    }

    #[test]
    fn parse_reversed_range_is_error() {
        assert!(PageRange::parse("5-3").is_err());
    }

    #[test]
    fn parse_empty_is_error() {
        assert!(PageRange::parse("").is_err());
    }

    #[test]
    fn all_pages() {
        let range = PageRange::all(5);
        for i in 0..5 {
            assert!(range.contains(i));
        }
        assert!(!range.contains(5));
    }
}
