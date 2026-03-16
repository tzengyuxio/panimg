//! Tests for PSD decoding support.
//!
//! These tests are only compiled when the `psd` feature is enabled.

#![cfg(feature = "psd")]

use panimg_core::codec::CodecRegistry;
use panimg_core::format::ImageFormat;
use std::path::Path;

fn test_psd_path() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/test.psd"
    ))
}

#[test]
fn detect_psd_magic_bytes() {
    let data = b"8BPS\x00\x01rest of file";
    assert_eq!(ImageFormat::from_bytes(data), Some(ImageFormat::Psd));
}

#[test]
fn detect_psd_from_extension() {
    assert_eq!(ImageFormat::from_extension("psd"), Some(ImageFormat::Psd));
    assert_eq!(ImageFormat::from_extension("PSD"), Some(ImageFormat::Psd));
}

#[test]
fn psd_format_properties() {
    assert_eq!(ImageFormat::Psd.extension(), "psd");
    assert_eq!(ImageFormat::Psd.mime_type(), "image/vnd.adobe.photoshop");
    assert_eq!(ImageFormat::Psd.to_image_format(), None);
    assert_eq!(ImageFormat::Psd.to_string(), "PSD");
}

#[test]
fn psd_cannot_encode() {
    assert!(!ImageFormat::Psd.can_encode());
}

#[test]
fn psd_can_decode_with_feature() {
    assert!(ImageFormat::Psd.can_decode());
}

#[test]
fn psd_in_all_list() {
    assert!(ImageFormat::all().contains(&ImageFormat::Psd));
}

#[test]
fn decode_psd_file() {
    let img = CodecRegistry::decode(test_psd_path()).expect("should decode PSD");
    assert_eq!(img.width(), 64);
    assert_eq!(img.height(), 64);
}

#[test]
fn get_psd_info() {
    let data = std::fs::read(test_psd_path()).unwrap();
    let info = panimg_core::psd::get_psd_info(&data).expect("should parse PSD info");
    assert_eq!(info.width, 64);
    assert_eq!(info.height, 64);
}
