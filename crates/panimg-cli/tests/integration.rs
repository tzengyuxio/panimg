use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;
use tempfile::TempDir;

fn panimg() -> Command {
    Command::cargo_bin("panimg").unwrap()
}

/// Create a minimal 4x4 PNG test image in the given directory.
fn create_test_png(dir: &Path, name: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    let img = image::RgbaImage::from_fn(4, 4, |x, y| {
        image::Rgba([(x * 64) as u8, (y * 64) as u8, 128, 255])
    });
    img.save(&path).unwrap();
    path
}

// ---- Help & Capabilities ----

#[test]
fn help_shows_subcommands() {
    panimg()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("info"))
        .stdout(predicate::str::contains("convert"))
        .stdout(predicate::str::contains("resize"));
}

#[test]
fn capabilities_human() {
    panimg()
        .arg("--capabilities")
        .assert()
        .success()
        .stdout(predicate::str::contains("JPEG"))
        .stdout(predicate::str::contains("PNG"))
        .stdout(predicate::str::contains("WebP"));
}

#[test]
fn capabilities_json() {
    let output = panimg()
        .args(["--capabilities", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["commands"].is_array());
    assert!(json["formats"].is_array());
    assert!(json["version"].is_string());
}

#[test]
fn capabilities_includes_all_commands() {
    let output = panimg()
        .args(["--capabilities", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let commands: Vec<&str> = json["commands"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert!(commands.contains(&"info"));
    assert!(commands.contains(&"convert"));
    assert!(commands.contains(&"resize"));
    assert!(commands.contains(&"crop"));
    assert!(commands.contains(&"rotate"));
    assert!(commands.contains(&"flip"));
    assert!(commands.contains(&"auto-orient"));
    assert!(commands.contains(&"grayscale"));
    assert!(commands.contains(&"invert"));
    assert!(commands.contains(&"brightness"));
    assert!(commands.contains(&"contrast"));
    assert!(commands.contains(&"hue-rotate"));
}

// ---- Info Command ----

#[test]
fn info_png_human() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");

    panimg()
        .args(["info", img_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("PNG"))
        .stdout(predicate::str::contains("4x4"));
}

#[test]
fn info_png_json() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");

    let output = panimg()
        .args(["info", img_path.to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["width"], 4);
    assert_eq!(json["height"], 4);
    assert_eq!(json["format"], "png");
}

#[test]
fn info_json_fields_filter() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");

    let output = panimg()
        .args([
            "info",
            img_path.to_str().unwrap(),
            "--format",
            "json",
            "--fields",
            "width,height",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let obj = json.as_object().unwrap();
    assert_eq!(obj.len(), 2);
    assert_eq!(json["width"], 4);
    assert_eq!(json["height"], 4);
}

#[test]
fn info_missing_file_error() {
    panimg()
        .args(["info", "nonexistent.png"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("file not found"));
}

#[test]
fn info_missing_file_json_error() {
    let output = panimg()
        .args(["info", "nonexistent.png", "--format", "json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let json: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(json["error"], "file_not_found");
}

#[test]
fn info_schema() {
    let output = panimg().args(["info", "--schema"]).output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "info");
    assert!(json["params"].is_array());
}

#[test]
fn info_human_fields_filter() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");

    panimg()
        .args([
            "info",
            img_path.to_str().unwrap(),
            "--fields",
            "width,height",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Dimensions:"))
        .stdout(predicate::str::contains("File:").not());
}

// ---- Convert Command ----

#[test]
fn convert_positional_output() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("test.bmp");

    panimg()
        .args([
            "convert",
            img_path.to_str().unwrap(),
            out_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn convert_png_to_bmp() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("test.bmp");

    panimg()
        .args([
            "convert",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn convert_png_to_jpeg() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("test.jpg");

    panimg()
        .args([
            "convert",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--quality",
            "90",
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn convert_dry_run_no_file() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("test.webp");

    panimg()
        .args([
            "convert",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Would convert"));

    assert!(!out_path.exists());
}

#[test]
fn convert_dry_run_json() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("test.bmp");

    let output = panimg()
        .args([
            "convert",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--dry-run",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["from_format"].is_string());
    assert!(json["to_format"].is_string());
    assert!(!out_path.exists());
}

#[test]
fn convert_missing_file_error() {
    panimg()
        .args(["convert", "nonexistent.png", "-o", "out.bmp"])
        .assert()
        .code(2);
}

#[test]
fn convert_output_exists_error() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = create_test_png(dir.path(), "existing.bmp");

    panimg()
        .args([
            "convert",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .code(3);
}

#[test]
fn convert_schema() {
    let output = panimg().args(["convert", "--schema"]).output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "convert");
}

// ---- Resize Command ----

#[test]
fn resize_width_only() {
    let dir = TempDir::new().unwrap();
    let img_path = dir.path().join("test.png");
    let img = image::RgbaImage::from_fn(100, 200, |_, _| image::Rgba([128, 128, 128, 255]));
    img.save(&img_path).unwrap();
    let out_path = dir.path().join("small.png");

    panimg()
        .args([
            "resize",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--width",
            "50",
        ])
        .assert()
        .success();

    let result = image::open(&out_path).unwrap();
    assert_eq!(result.width(), 50);
    assert_eq!(result.height(), 100);
}

#[test]
fn resize_height_only() {
    let dir = TempDir::new().unwrap();
    let img_path = dir.path().join("test.png");
    let img = image::RgbaImage::from_fn(200, 100, |_, _| image::Rgba([128, 128, 128, 255]));
    img.save(&img_path).unwrap();
    let out_path = dir.path().join("small.png");

    panimg()
        .args([
            "resize",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--height",
            "50",
        ])
        .assert()
        .success();

    let result = image::open(&out_path).unwrap();
    assert_eq!(result.height(), 50);
    assert_eq!(result.width(), 100);
}

#[test]
fn resize_dry_run() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("small.png");

    panimg()
        .args([
            "resize",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--width",
            "2",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Would resize"));

    assert!(!out_path.exists());
}

#[test]
fn resize_dry_run_json() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("small.png");

    let output = panimg()
        .args([
            "resize",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--width",
            "2",
            "--dry-run",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["steps"].is_array());
    assert!(!out_path.exists());
}

#[test]
fn resize_schema() {
    let output = panimg().args(["resize", "--schema"]).output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "resize");
    assert!(json["params"].is_array());
}

#[test]
fn resize_missing_dimensions_error() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");

    panimg()
        .args(["resize", img_path.to_str().unwrap(), "-o", "out.png"])
        .assert()
        .code(5);
}

#[test]
fn resize_to_jpeg_with_quality() {
    let dir = TempDir::new().unwrap();
    let img_path = dir.path().join("test.png");
    let img = image::RgbaImage::from_fn(100, 100, |_, _| image::Rgba([128, 128, 128, 255]));
    img.save(&img_path).unwrap();
    let out_path = dir.path().join("small.jpg");

    panimg()
        .args([
            "resize",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--width",
            "50",
            "--quality",
            "80",
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

// ---- Crop Command ----

#[test]
fn crop_basic() {
    let dir = TempDir::new().unwrap();
    let img_path = dir.path().join("test.png");
    let img = image::RgbaImage::from_fn(100, 100, |_, _| image::Rgba([128, 128, 128, 255]));
    img.save(&img_path).unwrap();
    let out_path = dir.path().join("cropped.png");

    panimg()
        .args([
            "crop",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--x",
            "10",
            "--y",
            "10",
            "--width",
            "50",
            "--height",
            "50",
        ])
        .assert()
        .success();

    let result = image::open(&out_path).unwrap();
    assert_eq!(result.width(), 50);
    assert_eq!(result.height(), 50);
}

#[test]
fn crop_out_of_bounds_error() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");

    panimg()
        .args([
            "crop",
            img_path.to_str().unwrap(),
            "-o",
            "out.png",
            "--width",
            "100",
            "--height",
            "100",
        ])
        .assert()
        .code(5);
}

#[test]
fn crop_dry_run_json() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("cropped.png");

    let output = panimg()
        .args([
            "crop",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--width",
            "2",
            "--height",
            "2",
            "--dry-run",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["steps"].is_array());
    assert!(!out_path.exists());
}

#[test]
fn crop_schema() {
    let output = panimg().args(["crop", "--schema"]).output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "crop");
}

// ---- Rotate Command ----

#[test]
fn rotate_90() {
    let dir = TempDir::new().unwrap();
    let img_path = dir.path().join("test.png");
    let img = image::RgbaImage::from_fn(100, 50, |_, _| image::Rgba([128, 128, 128, 255]));
    img.save(&img_path).unwrap();
    let out_path = dir.path().join("rotated.png");

    panimg()
        .args([
            "rotate",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--angle",
            "90",
        ])
        .assert()
        .success();

    let result = image::open(&out_path).unwrap();
    assert_eq!(result.width(), 50);
    assert_eq!(result.height(), 100);
}

#[test]
fn rotate_left_alias() {
    let dir = TempDir::new().unwrap();
    let img_path = dir.path().join("test.png");
    let img = image::RgbaImage::from_fn(100, 50, |_, _| image::Rgba([128, 128, 128, 255]));
    img.save(&img_path).unwrap();
    let out_path = dir.path().join("rotated.png");

    panimg()
        .args([
            "rotate",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--angle",
            "left",
        ])
        .assert()
        .success();

    let result = image::open(&out_path).unwrap();
    assert_eq!(result.width(), 50);
    assert_eq!(result.height(), 100);
}

#[test]
fn rotate_schema() {
    let output = panimg().args(["rotate", "--schema"]).output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "rotate");
}

// ---- Flip Command ----

#[test]
fn flip_horizontal() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("flipped.png");

    panimg()
        .args([
            "flip",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--direction",
            "horizontal",
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn flip_vertical() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("flipped.png");

    panimg()
        .args([
            "flip",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--direction",
            "v",
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn flip_schema() {
    let output = panimg().args(["flip", "--schema"]).output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "flip");
}

// ---- Auto-Orient Command ----

#[test]
fn auto_orient_no_exif() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("oriented.png");

    panimg()
        .args([
            "auto-orient",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    let result = image::open(&out_path).unwrap();
    assert_eq!(result.width(), 4);
    assert_eq!(result.height(), 4);
}

#[test]
fn auto_orient_schema() {
    let output = panimg().args(["auto-orient", "--schema"]).output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "auto-orient");
}

// ---- Grayscale Command ----

#[test]
fn grayscale_basic() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("gray.png");

    panimg()
        .args([
            "grayscale",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(out_path.exists());
    let result = image::open(&out_path).unwrap();
    assert_eq!(result.width(), 4);
}

#[test]
fn grayscale_schema() {
    let output = panimg().args(["grayscale", "--schema"]).output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "grayscale");
}

#[test]
fn grayscale_dry_run() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("gray.png");

    panimg()
        .args([
            "grayscale",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Would"));

    assert!(!out_path.exists());
}

// ---- Invert Command ----

#[test]
fn invert_basic() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("inverted.png");

    panimg()
        .args([
            "invert",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn invert_schema() {
    let output = panimg().args(["invert", "--schema"]).output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "invert");
}

// ---- Brightness Command ----

#[test]
fn brightness_increase() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("bright.png");

    panimg()
        .args([
            "brightness",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--value",
            "30",
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn brightness_schema() {
    let output = panimg().args(["brightness", "--schema"]).output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "brightness");
}

#[test]
fn brightness_missing_value_error() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");

    panimg()
        .args([
            "brightness",
            img_path.to_str().unwrap(),
            "-o",
            "out.png",
        ])
        .assert()
        .code(5);
}

// ---- Contrast Command ----

#[test]
fn contrast_increase() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("contrast.png");

    panimg()
        .args([
            "contrast",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--value",
            "20",
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn contrast_schema() {
    let output = panimg().args(["contrast", "--schema"]).output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "contrast");
}

// ---- Hue-Rotate Command ----

#[test]
fn hue_rotate_basic() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("hue.png");

    panimg()
        .args([
            "hue-rotate",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--degrees",
            "90",
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn hue_rotate_schema() {
    let output = panimg()
        .args(["hue-rotate", "--schema"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "hue-rotate");
}

#[test]
fn hue_rotate_json_output() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("hue.png");

    let output = panimg()
        .args([
            "hue-rotate",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--degrees",
            "120",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["degrees"], 120);
}
