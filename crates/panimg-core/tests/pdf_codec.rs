//! Tests for PDF decoding support.
//!
//! These tests are only compiled when the `pdf` feature is enabled.

#![cfg(feature = "pdf")]

use panimg_core::codec::{CodecRegistry, DecodeOptions};
use panimg_core::format::ImageFormat;
use std::io::Write;
use tempfile::NamedTempFile;

/// A minimal valid PDF that draws a red rectangle on a white page.
/// Page size: 200x100 points (at 72 DPI).
fn minimal_pdf() -> Vec<u8> {
    let content = b"1 0 0 rg 10 10 180 80 re f";
    let content_len = content.len();

    let mut pdf = Vec::new();
    write!(pdf, "%PDF-1.4\n").unwrap();

    // Object 1: Catalog
    let obj1_offset = pdf.len();
    write!(pdf, "1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n").unwrap();

    // Object 2: Pages
    let obj2_offset = pdf.len();
    write!(
        pdf,
        "2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n"
    )
    .unwrap();

    // Object 3: Page
    let obj3_offset = pdf.len();
    write!(
        pdf,
        "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 100] /Contents 4 0 R >>\nendobj\n"
    )
    .unwrap();

    // Object 4: Content stream
    let obj4_offset = pdf.len();
    write!(pdf, "4 0 obj\n<< /Length {content_len} >>\nstream\n").unwrap();
    pdf.extend_from_slice(content);
    write!(pdf, "\nendstream\nendobj\n").unwrap();

    // Cross-reference table
    let xref_offset = pdf.len();
    write!(pdf, "xref\n0 5\n").unwrap();
    write!(pdf, "0000000000 65535 f \n").unwrap();
    write!(pdf, "{:010} 00000 n \n", obj1_offset).unwrap();
    write!(pdf, "{:010} 00000 n \n", obj2_offset).unwrap();
    write!(pdf, "{:010} 00000 n \n", obj3_offset).unwrap();
    write!(pdf, "{:010} 00000 n \n", obj4_offset).unwrap();

    // Trailer
    write!(pdf, "trailer\n<< /Size 5 /Root 1 0 R >>\n").unwrap();
    write!(pdf, "startxref\n{xref_offset}\n%%EOF\n").unwrap();

    pdf
}

#[test]
fn detect_pdf_from_bytes() {
    let data = minimal_pdf();
    assert_eq!(ImageFormat::from_bytes(&data), Some(ImageFormat::Pdf));
}

#[test]
fn decode_pdf_first_page() {
    let data = minimal_pdf();
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&data).unwrap();
    tmp.flush().unwrap();

    let img = CodecRegistry::decode(tmp.path()).expect("should decode PDF");
    // At 150 DPI (default), 200pt page = 200 * 150/72 ~= 416 px wide
    // 100pt page = 100 * 150/72 ~= 208 px tall
    assert!(img.width() > 0);
    assert!(img.height() > 0);
    // Verify approximate dimensions at default 150 DPI
    let expected_width = (200.0 * 150.0 / 72.0) as u32;
    let expected_height = (100.0 * 150.0 / 72.0) as u32;
    // Allow some rounding tolerance
    assert!((img.width() as i32 - expected_width as i32).unsigned_abs() <= 2);
    assert!((img.height() as i32 - expected_height as i32).unsigned_abs() <= 2);
}

#[test]
fn decode_pdf_custom_dpi() {
    let data = minimal_pdf();
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&data).unwrap();
    tmp.flush().unwrap();

    let opts = DecodeOptions { dpi: 72.0 };
    let img = CodecRegistry::decode_with_options(tmp.path(), &opts).expect("should decode PDF");
    // At 72 DPI (1:1 scale), 200pt = 200px, 100pt = 100px
    assert!((img.width() as i32 - 200).unsigned_abs() <= 2);
    assert!((img.height() as i32 - 100).unsigned_abs() <= 2);
}

#[test]
fn decode_pdf_high_dpi() {
    let data = minimal_pdf();
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&data).unwrap();
    tmp.flush().unwrap();

    let opts = DecodeOptions { dpi: 300.0 };
    let img = CodecRegistry::decode_with_options(tmp.path(), &opts).expect("should decode PDF");
    // At 300 DPI, 200pt = 200*300/72 ~= 833px
    let expected_width = (200.0 * 300.0 / 72.0) as u32;
    assert!((img.width() as i32 - expected_width as i32).unsigned_abs() <= 2);
}

#[test]
fn pdf_format_in_all_list() {
    assert!(ImageFormat::all().contains(&ImageFormat::Pdf));
}

#[test]
fn pdf_cannot_encode() {
    assert!(!ImageFormat::Pdf.can_encode());
}
