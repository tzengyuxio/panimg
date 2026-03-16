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

    let opts = DecodeOptions {
        dpi: 72.0,
        page: None,
    };
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

    let opts = DecodeOptions {
        dpi: 300.0,
        page: None,
    };
    let img = CodecRegistry::decode_with_options(tmp.path(), &opts).expect("should decode PDF");
    // At 300 DPI, 200pt = 200*300/72 ~= 833px
    let expected_width = (200.0 * 300.0 / 72.0) as u32;
    assert!((img.width() as i32 - expected_width as i32).unsigned_abs() <= 2);
}

/// A 3-page PDF. Each page is 200x100pt with a different colored rectangle:
/// Page 1: red, Page 2: green, Page 3: blue.
fn multi_page_pdf() -> Vec<u8> {
    let contents = [
        b"1 0 0 rg 10 10 180 80 re f" as &[u8], // red
        b"0 1 0 rg 10 10 180 80 re f",          // green
        b"0 0 1 rg 10 10 180 80 re f",          // blue
    ];

    let mut pdf = Vec::new();
    write!(pdf, "%PDF-1.4\n").unwrap();

    // Object 1: Catalog
    let obj1_offset = pdf.len();
    write!(pdf, "1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n").unwrap();

    // Object 2: Pages (3 kids)
    let obj2_offset = pdf.len();
    write!(
        pdf,
        "2 0 obj\n<< /Type /Pages /Kids [3 0 R 5 0 R 7 0 R] /Count 3 >>\nendobj\n"
    )
    .unwrap();

    // Pages: obj 3,4 / 5,6 / 7,8
    let mut offsets = Vec::new();
    let page_objs = [(3, 4), (5, 6), (7, 8)];
    for (i, &(page_obj, content_obj)) in page_objs.iter().enumerate() {
        let page_offset = pdf.len();
        offsets.push(page_offset);
        write!(
            pdf,
            "{page_obj} 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 100] /Contents {content_obj} 0 R >>\nendobj\n"
        )
        .unwrap();

        let content_offset = pdf.len();
        offsets.push(content_offset);
        let content = contents[i];
        write!(
            pdf,
            "{content_obj} 0 obj\n<< /Length {} >>\nstream\n",
            content.len()
        )
        .unwrap();
        pdf.extend_from_slice(content);
        write!(pdf, "\nendstream\nendobj\n").unwrap();
    }

    // Cross-reference table: 0 + 1(catalog) + 2(pages) + 3,4,5,6,7,8 = 9 entries
    let xref_offset = pdf.len();
    write!(pdf, "xref\n0 9\n").unwrap();
    write!(pdf, "0000000000 65535 f \n").unwrap();
    write!(pdf, "{:010} 00000 n \n", obj1_offset).unwrap();
    write!(pdf, "{:010} 00000 n \n", obj2_offset).unwrap();
    for off in &offsets {
        write!(pdf, "{:010} 00000 n \n", off).unwrap();
    }

    write!(pdf, "trailer\n<< /Size 9 /Root 1 0 R >>\n").unwrap();
    write!(pdf, "startxref\n{xref_offset}\n%%EOF\n").unwrap();

    pdf
}

#[test]
fn get_pdf_info_page_count() {
    let data = multi_page_pdf();
    let info = panimg_core::pdf::get_pdf_info(&data).expect("should get PDF info");
    assert_eq!(info.page_count, 3);
}

#[test]
fn get_pdf_info_single_page() {
    let data = minimal_pdf();
    let info = panimg_core::pdf::get_pdf_info(&data).expect("should get PDF info");
    assert_eq!(info.page_count, 1);
}

#[test]
fn decode_pdf_specific_page() {
    let data = multi_page_pdf();
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&data).unwrap();
    tmp.flush().unwrap();

    // Decode page 2 (0-based index 1)
    let opts = DecodeOptions {
        dpi: 72.0,
        page: Some(1),
    };
    let img = CodecRegistry::decode_with_options(tmp.path(), &opts).expect("should decode page 2");
    assert!(img.width() > 0);
    assert!(img.height() > 0);
}

#[test]
fn decode_pdf_page_out_of_range() {
    let data = multi_page_pdf();
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&data).unwrap();
    tmp.flush().unwrap();

    let opts = DecodeOptions {
        dpi: 72.0,
        page: Some(99),
    };
    let result = CodecRegistry::decode_with_options(tmp.path(), &opts);
    assert!(result.is_err());
}

#[test]
fn for_each_page_all() {
    use panimg_core::pdf::PageRange;

    let data = multi_page_pdf();
    let range = PageRange::all(3);
    let mut pages_visited = Vec::new();

    let total = panimg_core::pdf::for_each_page(&data, &range, 72.0, |info, img| {
        assert!(img.width() > 0);
        pages_visited.push(info.index);
        Ok(true)
    })
    .expect("should iterate pages");

    assert_eq!(total, 3);
    assert_eq!(pages_visited, vec![0, 1, 2]);
}

#[test]
fn for_each_page_range() {
    use panimg_core::pdf::PageRange;

    let data = multi_page_pdf();
    let range = PageRange::parse("1,3").unwrap(); // Pages 1 and 3 (0-based: 0, 2)
    let mut pages_visited = Vec::new();

    let total = panimg_core::pdf::for_each_page(&data, &range, 72.0, |info, _img| {
        pages_visited.push(info.index);
        Ok(true)
    })
    .expect("should iterate selected pages");

    assert_eq!(total, 3);
    assert_eq!(pages_visited, vec![0, 2]);
}

#[test]
fn for_each_page_out_of_range_skipped() {
    use panimg_core::pdf::PageRange;

    let data = multi_page_pdf();
    // Request pages 1-5 but PDF only has 3 pages — extra indices should be silently skipped
    let range = PageRange::parse("1-5").unwrap();
    let mut pages_visited = Vec::new();

    let total = panimg_core::pdf::for_each_page(&data, &range, 72.0, |info, _img| {
        pages_visited.push(info.index);
        Ok(true)
    })
    .expect("should iterate available pages");

    assert_eq!(total, 3);
    assert_eq!(pages_visited, vec![0, 1, 2]);
}

#[test]
fn pdf_format_in_all_list() {
    assert!(ImageFormat::all().contains(&ImageFormat::Pdf));
}

#[test]
fn pdf_cannot_encode() {
    assert!(!ImageFormat::Pdf.can_encode());
}
