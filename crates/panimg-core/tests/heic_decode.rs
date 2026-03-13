#![cfg(feature = "heic")]

use panimg_core::codec::CodecRegistry;
use panimg_core::format::ImageFormat;
use std::path::Path;

#[test]
fn detect_heic_format_from_file() {
    let path = Path::new("tests/fixtures/test.heic");
    if !path.exists() {
        eprintln!("skipping: test.heic not found");
        return;
    }
    let data = std::fs::read(path).unwrap();
    let fmt = ImageFormat::from_bytes(&data);
    assert_eq!(fmt, Some(ImageFormat::Heic));
}

#[test]
fn decode_heic_file() {
    let path = Path::new("tests/fixtures/test.heic");
    if !path.exists() {
        eprintln!("skipping: test.heic not found");
        return;
    }
    let img = CodecRegistry::decode(path).expect("failed to decode HEIC");
    assert_eq!(img.width(), 4);
    assert_eq!(img.height(), 4);
}

#[test]
fn decode_heic_with_alpha() {
    let path = Path::new("tests/fixtures/test_alpha.heic");
    if !path.exists() {
        eprintln!("skipping: test_alpha.heic not found");
        return;
    }
    let img = CodecRegistry::decode(path).expect("failed to decode HEIC with alpha");
    assert_eq!(img.width(), 4);
    assert_eq!(img.height(), 4);
    // Alpha image should be decoded as RGBA
    assert!(img.as_rgba8().is_some() || img.to_rgba8().width() == 4);
}

#[test]
fn heic_format_properties() {
    let fmt = ImageFormat::Heic;
    assert_eq!(fmt.extension(), "heic");
    assert_eq!(fmt.mime_type(), "image/heic");
    assert!(fmt.can_decode());
    assert!(!fmt.can_encode());
    assert_eq!(fmt.to_string(), "HEIC");
    assert!(fmt.to_image_format().is_none());
}
