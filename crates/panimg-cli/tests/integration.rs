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
    assert!(commands.contains(&"blur"));
    assert!(commands.contains(&"sharpen"));
    assert!(commands.contains(&"batch"));
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
        .args(["brightness", img_path.to_str().unwrap(), "-o", "out.png"])
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
    let output = panimg().args(["hue-rotate", "--schema"]).output().unwrap();
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

// ---- Batch Command ----

#[test]
fn batch_convert_to_bmp() {
    let dir = TempDir::new().unwrap();
    create_test_png(dir.path(), "a.png");
    create_test_png(dir.path(), "b.png");
    let out_dir = dir.path().join("out");

    let pattern = dir.path().join("*.png");

    panimg()
        .args([
            "batch",
            "convert",
            pattern.to_str().unwrap(),
            "--output-dir",
            out_dir.to_str().unwrap(),
            "--to",
            "bmp",
        ])
        .assert()
        .success();

    assert!(out_dir.join("a.bmp").exists());
    assert!(out_dir.join("b.bmp").exists());
}

#[test]
fn batch_grayscale() {
    let dir = TempDir::new().unwrap();
    create_test_png(dir.path(), "img1.png");
    create_test_png(dir.path(), "img2.png");
    let out_dir = dir.path().join("gray");

    let pattern = dir.path().join("*.png");

    panimg()
        .args([
            "batch",
            "grayscale",
            pattern.to_str().unwrap(),
            "--output-dir",
            out_dir.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(out_dir.join("img1.png").exists());
    assert!(out_dir.join("img2.png").exists());
}

#[test]
fn batch_resize() {
    let dir = TempDir::new().unwrap();
    let img = image::RgbaImage::from_fn(100, 200, |_, _| image::Rgba([128, 128, 128, 255]));
    img.save(dir.path().join("big.png")).unwrap();
    let out_dir = dir.path().join("thumbs");

    let pattern = dir.path().join("*.png");

    panimg()
        .args([
            "batch",
            "resize",
            pattern.to_str().unwrap(),
            "--output-dir",
            out_dir.to_str().unwrap(),
            "--width",
            "50",
        ])
        .assert()
        .success();

    let result = image::open(out_dir.join("big.png")).unwrap();
    assert_eq!(result.width(), 50);
    assert_eq!(result.height(), 100);
}

#[test]
fn batch_dry_run() {
    let dir = TempDir::new().unwrap();
    create_test_png(dir.path(), "test.png");
    let out_dir = dir.path().join("out");

    let pattern = dir.path().join("*.png");

    panimg()
        .args([
            "batch",
            "grayscale",
            pattern.to_str().unwrap(),
            "--output-dir",
            out_dir.to_str().unwrap(),
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Would"));

    assert!(!out_dir.exists());
}

#[test]
fn batch_dry_run_json() {
    let dir = TempDir::new().unwrap();
    create_test_png(dir.path(), "a.png");
    create_test_png(dir.path(), "b.png");
    let out_dir = dir.path().join("out");

    let pattern = dir.path().join("*.png");

    let output = panimg()
        .args([
            "batch",
            "invert",
            pattern.to_str().unwrap(),
            "--output-dir",
            out_dir.to_str().unwrap(),
            "--dry-run",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["total_files"], 2);
    assert!(json["files"].is_array());
}

#[test]
fn batch_json_output() {
    let dir = TempDir::new().unwrap();
    create_test_png(dir.path(), "x.png");
    let out_dir = dir.path().join("out");

    let pattern = dir.path().join("*.png");

    let output = panimg()
        .args([
            "batch",
            "grayscale",
            pattern.to_str().unwrap(),
            "--output-dir",
            out_dir.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["total"], 1);
    assert_eq!(json["succeeded"], 1);
    assert_eq!(json["failed"], 0);
}

#[test]
fn batch_no_match_error() {
    let dir = TempDir::new().unwrap();
    let pattern = dir.path().join("*.doesnotexist");

    panimg()
        .args([
            "batch",
            "grayscale",
            pattern.to_str().unwrap(),
            "--output-dir",
            "/tmp/out",
        ])
        .assert()
        .code(5);
}

#[test]
fn batch_output_template() {
    let dir = TempDir::new().unwrap();
    create_test_png(dir.path(), "photo.png");
    let out_dir = dir.path().join("out");
    std::fs::create_dir(&out_dir).unwrap();

    let pattern = dir.path().join("*.png");
    let template = format!("{}/{{stem}}_gray.{{ext}}", out_dir.to_str().unwrap());

    panimg()
        .args([
            "batch",
            "grayscale",
            pattern.to_str().unwrap(),
            "--output-template",
            &template,
        ])
        .assert()
        .success();

    assert!(out_dir.join("photo_gray.png").exists());
}

// ---- Blur Command ----

#[test]
fn blur_basic() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("blurred.png");

    panimg()
        .args([
            "blur",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--sigma",
            "2.0",
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn blur_schema() {
    let output = panimg().args(["blur", "--schema"]).output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "blur");
}

#[test]
fn blur_missing_sigma_error() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");

    panimg()
        .args(["blur", img_path.to_str().unwrap(), "-o", "out.png"])
        .assert()
        .code(5);
}

#[test]
fn blur_json_output() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("blurred.png");

    let output = panimg()
        .args([
            "blur",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--sigma",
            "1.5",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["sigma"], 1.5);
}

// ---- Sharpen Command ----

#[test]
fn sharpen_basic() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("sharp.png");

    panimg()
        .args([
            "sharpen",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--sigma",
            "1.0",
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn sharpen_with_threshold() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("sharp.png");

    panimg()
        .args([
            "sharpen",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--sigma",
            "2.0",
            "--threshold",
            "10",
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn sharpen_schema() {
    let output = panimg().args(["sharpen", "--schema"]).output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "sharpen");
}

// ---- Edge Detect ----

#[test]
fn edge_detect_produces_output() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("edges.png");

    panimg()
        .args([
            "edge-detect",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn edge_detect_json_output() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("edges.png");

    let output = panimg()
        .args([
            "edge-detect",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["input"], img_path.to_str().unwrap());
    assert!(json["output_size"].as_u64().unwrap() > 0);
}

#[test]
fn edge_detect_dry_run() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("edges.png");

    panimg()
        .args([
            "edge-detect",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--dry-run",
            "--format",
            "json",
        ])
        .assert()
        .success();

    assert!(!out_path.exists());
}

#[test]
fn edge_detect_schema() {
    let output = panimg().args(["edge-detect", "--schema"]).output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "edge-detect");
}

#[test]
fn edge_detect_missing_input() {
    panimg().args(["edge-detect"]).assert().failure();
}

#[test]
fn edge_detect_positional_output() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("edges.png");

    panimg()
        .args([
            "edge-detect",
            img_path.to_str().unwrap(),
            out_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

// ---- Emboss ----

#[test]
fn emboss_produces_output() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("embossed.png");

    panimg()
        .args([
            "emboss",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn emboss_json_output() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("embossed.png");

    let output = panimg()
        .args([
            "emboss",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["input"], img_path.to_str().unwrap());
    assert!(json["output_size"].as_u64().unwrap() > 0);
}

#[test]
fn emboss_dry_run() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("embossed.png");

    panimg()
        .args([
            "emboss",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--dry-run",
            "--format",
            "json",
        ])
        .assert()
        .success();

    assert!(!out_path.exists());
}

#[test]
fn emboss_schema() {
    let output = panimg().args(["emboss", "--schema"]).output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "emboss");
}

#[test]
fn emboss_missing_input() {
    panimg().args(["emboss"]).assert().failure();
}

#[test]
fn emboss_positional_output() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("embossed.png");

    panimg()
        .args([
            "emboss",
            img_path.to_str().unwrap(),
            out_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

// ---- Batch edge-detect & emboss ----

#[test]
fn batch_edge_detect() {
    let dir = TempDir::new().unwrap();
    create_test_png(dir.path(), "a.png");
    create_test_png(dir.path(), "b.png");
    let out_dir = dir.path().join("out");

    let pattern = dir.path().join("*.png");
    panimg()
        .args([
            "batch",
            "edge-detect",
            pattern.to_str().unwrap(),
            "--output-dir",
            out_dir.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(out_dir.join("a.png").exists());
    assert!(out_dir.join("b.png").exists());
}

#[test]
fn batch_emboss() {
    let dir = TempDir::new().unwrap();
    create_test_png(dir.path(), "a.png");
    create_test_png(dir.path(), "b.png");
    let out_dir = dir.path().join("out");

    let pattern = dir.path().join("*.png");
    panimg()
        .args([
            "batch",
            "emboss",
            pattern.to_str().unwrap(),
            "--output-dir",
            out_dir.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(out_dir.join("a.png").exists());
    assert!(out_dir.join("b.png").exists());
}

// ---- Overlay ----

/// Create a small overlay image (red square with alpha).
fn create_overlay_png(dir: &Path, name: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    let img = image::RgbaImage::from_fn(2, 2, |_, _| image::Rgba([255, 0, 0, 200]));
    img.save(&path).unwrap();
    path
}

#[test]
fn overlay_basic() {
    let dir = TempDir::new().unwrap();
    let base = create_test_png(dir.path(), "base.png");
    let layer = create_overlay_png(dir.path(), "layer.png");
    let out_path = dir.path().join("result.png");

    panimg()
        .args([
            "overlay",
            base.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--layer",
            layer.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn overlay_with_position() {
    let dir = TempDir::new().unwrap();
    let base = create_test_png(dir.path(), "base.png");
    let layer = create_overlay_png(dir.path(), "layer.png");
    let out_path = dir.path().join("result.png");

    panimg()
        .args([
            "overlay",
            base.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--layer",
            layer.to_str().unwrap(),
            "--position",
            "center",
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn overlay_with_opacity() {
    let dir = TempDir::new().unwrap();
    let base = create_test_png(dir.path(), "base.png");
    let layer = create_overlay_png(dir.path(), "layer.png");
    let out_path = dir.path().join("result.png");

    panimg()
        .args([
            "overlay",
            base.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--layer",
            layer.to_str().unwrap(),
            "--opacity",
            "0.5",
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn overlay_with_xy_offset() {
    let dir = TempDir::new().unwrap();
    let base = create_test_png(dir.path(), "base.png");
    let layer = create_overlay_png(dir.path(), "layer.png");
    let out_path = dir.path().join("result.png");

    panimg()
        .args([
            "overlay",
            base.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--layer",
            layer.to_str().unwrap(),
            "--x",
            "1",
            "--y",
            "1",
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn overlay_tiled() {
    let dir = TempDir::new().unwrap();
    let base = create_test_png(dir.path(), "base.png");
    let layer = create_overlay_png(dir.path(), "layer.png");
    let out_path = dir.path().join("result.png");

    panimg()
        .args([
            "overlay",
            base.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--layer",
            layer.to_str().unwrap(),
            "--tile",
            "--opacity",
            "0.3",
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn overlay_json_output() {
    let dir = TempDir::new().unwrap();
    let base = create_test_png(dir.path(), "base.png");
    let layer = create_overlay_png(dir.path(), "layer.png");
    let out_path = dir.path().join("result.png");

    let output = panimg()
        .args([
            "overlay",
            base.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--layer",
            layer.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["opacity"], 1.0);
    assert!(json["output_size"].as_u64().unwrap() > 0);
}

#[test]
fn overlay_dry_run() {
    let dir = TempDir::new().unwrap();
    let base = create_test_png(dir.path(), "base.png");
    let layer = create_overlay_png(dir.path(), "layer.png");
    let out_path = dir.path().join("result.png");

    panimg()
        .args([
            "overlay",
            base.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--layer",
            layer.to_str().unwrap(),
            "--dry-run",
            "--format",
            "json",
        ])
        .assert()
        .success();

    assert!(!out_path.exists());
}

#[test]
fn overlay_schema() {
    let output = panimg().args(["overlay", "--schema"]).output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "overlay");
}

#[test]
fn overlay_missing_layer() {
    let dir = TempDir::new().unwrap();
    let base = create_test_png(dir.path(), "base.png");
    let out_path = dir.path().join("result.png");

    panimg()
        .args([
            "overlay",
            base.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .failure();
}

// ---- Trim ----

/// Create a test image with white border and colored center for trim testing.
fn create_bordered_png(dir: &Path, name: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    let img = image::RgbaImage::from_fn(8, 8, |x, y| {
        if x >= 2 && x < 6 && y >= 2 && y < 6 {
            image::Rgba([255, 0, 0, 255])
        } else {
            image::Rgba([255, 255, 255, 255])
        }
    });
    img.save(&path).unwrap();
    path
}

#[test]
fn trim_basic() {
    let dir = TempDir::new().unwrap();
    let img_path = create_bordered_png(dir.path(), "bordered.png");
    let out_path = dir.path().join("trimmed.png");

    panimg()
        .args([
            "trim",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(out_path.exists());
    let result = image::open(&out_path).unwrap();
    assert_eq!(result.width(), 4);
    assert_eq!(result.height(), 4);
}

#[test]
fn trim_with_tolerance() {
    let dir = TempDir::new().unwrap();
    let img_path = create_bordered_png(dir.path(), "bordered.png");
    let out_path = dir.path().join("trimmed.png");

    panimg()
        .args([
            "trim",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--tolerance",
            "0",
        ])
        .assert()
        .success();

    assert!(out_path.exists());
    let result = image::open(&out_path).unwrap();
    assert_eq!(result.width(), 4);
    assert_eq!(result.height(), 4);
}

#[test]
fn trim_json_output() {
    let dir = TempDir::new().unwrap();
    let img_path = create_bordered_png(dir.path(), "bordered.png");
    let out_path = dir.path().join("trimmed.png");

    let output = panimg()
        .args([
            "trim",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["original_width"], 8);
    assert_eq!(json["original_height"], 8);
    assert_eq!(json["trimmed_width"], 4);
    assert_eq!(json["trimmed_height"], 4);
}

#[test]
fn trim_dry_run() {
    let dir = TempDir::new().unwrap();
    let img_path = create_bordered_png(dir.path(), "bordered.png");
    let out_path = dir.path().join("trimmed.png");

    panimg()
        .args([
            "trim",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--dry-run",
            "--format",
            "json",
        ])
        .assert()
        .success();

    assert!(!out_path.exists());
}

#[test]
fn trim_schema() {
    let output = panimg().args(["trim", "--schema"]).output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "trim");
}

#[test]
fn trim_missing_input() {
    panimg().args(["trim"]).assert().failure();
}

#[test]
fn batch_trim() {
    let dir = TempDir::new().unwrap();
    create_bordered_png(dir.path(), "a.png");
    create_bordered_png(dir.path(), "b.png");
    let out_dir = dir.path().join("out");

    let pattern = dir.path().join("*.png");
    panimg()
        .args([
            "batch",
            "trim",
            pattern.to_str().unwrap(),
            "--output-dir",
            out_dir.to_str().unwrap(),
            "--tolerance",
            "0",
        ])
        .assert()
        .success();

    assert!(out_dir.join("a.png").exists());
    assert!(out_dir.join("b.png").exists());

    let result = image::open(out_dir.join("a.png")).unwrap();
    assert_eq!(result.width(), 4);
    assert_eq!(result.height(), 4);
}

// ---- Diff ----

#[test]
fn diff_identical_images() {
    let dir = TempDir::new().unwrap();
    let img_a = create_test_png(dir.path(), "a.png");

    let output = panimg()
        .args([
            "diff",
            img_a.to_str().unwrap(),
            img_a.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["identical"], true);
    assert_eq!(json["diff_pixels"], 0);
}

#[test]
fn diff_different_images_exits_1() {
    let dir = TempDir::new().unwrap();
    let img_a = create_test_png(dir.path(), "a.png");

    // Create a different image
    let img_b_path = dir.path().join("b.png");
    let img_b = image::RgbaImage::from_fn(4, 4, |_, _| image::Rgba([0, 0, 255, 255]));
    img_b.save(&img_b_path).unwrap();

    panimg()
        .args([
            "diff",
            img_a.to_str().unwrap(),
            img_b_path.to_str().unwrap(),
            "--format",
            "json",
        ])
        .assert()
        .code(1);
}

#[test]
fn diff_with_output_image() {
    let dir = TempDir::new().unwrap();
    let img_a = create_test_png(dir.path(), "a.png");
    let img_b_path = dir.path().join("b.png");
    let img_b = image::RgbaImage::from_fn(4, 4, |_, _| image::Rgba([0, 0, 255, 255]));
    img_b.save(&img_b_path).unwrap();
    let diff_path = dir.path().join("diff.png");

    panimg()
        .args([
            "diff",
            img_a.to_str().unwrap(),
            img_b_path.to_str().unwrap(),
            "-o",
            diff_path.to_str().unwrap(),
            "--format",
            "json",
        ])
        .assert()
        .code(1);

    assert!(diff_path.exists());
    let diff_img = image::open(&diff_path).unwrap();
    assert_eq!(diff_img.width(), 4);
    assert_eq!(diff_img.height(), 4);
}

#[test]
fn diff_with_threshold() {
    let dir = TempDir::new().unwrap();

    // Create two images with small differences
    let a_path = dir.path().join("a.png");
    let b_path = dir.path().join("b.png");
    let img_a = image::RgbaImage::from_pixel(4, 4, image::Rgba([100, 100, 100, 255]));
    let img_b = image::RgbaImage::from_pixel(4, 4, image::Rgba([105, 105, 105, 255]));
    img_a.save(&a_path).unwrap();
    img_b.save(&b_path).unwrap();

    // With threshold 10, should be identical
    let output = panimg()
        .args([
            "diff",
            a_path.to_str().unwrap(),
            b_path.to_str().unwrap(),
            "--threshold",
            "10",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["identical"], true);
}

#[test]
fn diff_dry_run() {
    let dir = TempDir::new().unwrap();
    let img_a = create_test_png(dir.path(), "a.png");

    panimg()
        .args([
            "diff",
            img_a.to_str().unwrap(),
            img_a.to_str().unwrap(),
            "--dry-run",
            "--format",
            "json",
        ])
        .assert()
        .success();
}

#[test]
fn diff_json_fields() {
    let dir = TempDir::new().unwrap();
    let img_a = create_test_png(dir.path(), "a.png");
    let img_b_path = dir.path().join("b.png");
    let img_b = image::RgbaImage::from_fn(4, 4, |_, _| image::Rgba([0, 0, 255, 255]));
    img_b.save(&img_b_path).unwrap();

    let output = panimg()
        .args([
            "diff",
            img_a.to_str().unwrap(),
            img_b_path.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.get("identical").is_some());
    assert!(json.get("diff_pixels").is_some());
    assert!(json.get("total_pixels").is_some());
    assert!(json.get("diff_percent").is_some());
    assert!(json.get("mae").is_some());
    assert!(json.get("dimensions_match").is_some());
    assert!(json.get("width_a").is_some());
}

#[test]
fn diff_missing_inputs() {
    panimg().args(["diff"]).assert().failure();
}

// ---- Pipeline ----

#[test]
fn pipeline_single_step() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("out.png");

    panimg()
        .args([
            "pipeline",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--steps",
            "grayscale",
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn pipeline_multi_steps() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("out.png");

    panimg()
        .args([
            "pipeline",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--steps",
            "grayscale | blur --sigma 1.0 | invert",
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn pipeline_resize_and_filter() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("out.png");

    panimg()
        .args([
            "pipeline",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--steps",
            "resize --width 2 | sharpen --sigma 0.5",
        ])
        .assert()
        .success();

    assert!(out_path.exists());
    let result = image::open(&out_path).unwrap();
    assert_eq!(result.width(), 2);
}

#[test]
fn pipeline_json_output() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("out.png");

    let output = panimg()
        .args([
            "pipeline",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--steps",
            "grayscale | invert",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["steps"], 2);
    assert!(json["output_size"].as_u64().unwrap() > 0);
}

#[test]
fn pipeline_dry_run() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("out.png");

    let output = panimg()
        .args([
            "pipeline",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--steps",
            "grayscale | blur --sigma 2.0",
            "--dry-run",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(!out_path.exists());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["steps"].as_array().unwrap().len(), 2);
}

#[test]
fn pipeline_recipe_file() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("out.png");
    let recipe_path = dir.path().join("recipe.json");

    std::fs::write(
        &recipe_path,
        r#"{"steps": [{"op": "grayscale"}, {"op": "blur", "sigma": 1.0}]}"#,
    )
    .unwrap();

    panimg()
        .args([
            "pipeline",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--recipe",
            recipe_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn pipeline_recipe_with_resize() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("out.png");
    let recipe_path = dir.path().join("recipe.json");

    std::fs::write(
        &recipe_path,
        r#"{"steps": [{"op": "resize", "width": 2}, {"op": "emboss"}]}"#,
    )
    .unwrap();

    panimg()
        .args([
            "pipeline",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--recipe",
            recipe_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(out_path.exists());
    let result = image::open(&out_path).unwrap();
    assert_eq!(result.width(), 2);
}

#[test]
fn pipeline_missing_steps_and_recipe() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("out.png");

    panimg()
        .args([
            "pipeline",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .failure();
}

#[test]
fn pipeline_invalid_step() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("out.png");

    panimg()
        .args([
            "pipeline",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--steps",
            "nonexistent",
        ])
        .assert()
        .failure();
}

#[test]
fn pipeline_positional_output() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("out.png");

    panimg()
        .args([
            "pipeline",
            img_path.to_str().unwrap(),
            out_path.to_str().unwrap(),
            "--steps",
            "grayscale",
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

// ---- Draw ----

#[test]
fn draw_filled_rect() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("drawn.png");

    panimg()
        .args([
            "draw",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--shape",
            "rect",
            "--x",
            "0",
            "--y",
            "0",
            "--width",
            "2",
            "--height",
            "2",
            "--color",
            "red",
            "--fill",
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn draw_circle() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("drawn.png");

    panimg()
        .args([
            "draw",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--shape",
            "circle",
            "--cx",
            "2",
            "--cy",
            "2",
            "--radius",
            "1",
            "--color",
            "green",
            "--fill",
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn draw_line() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("drawn.png");

    panimg()
        .args([
            "draw",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--shape",
            "line",
            "--x1",
            "0",
            "--y1",
            "0",
            "--x2",
            "3",
            "--y2",
            "3",
            "--color",
            "255,0,0",
            "--thickness",
            "1",
        ])
        .assert()
        .success();

    assert!(out_path.exists());
}

#[test]
fn draw_json_output() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("drawn.png");

    let output = panimg()
        .args([
            "draw",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--shape",
            "rect",
            "--width",
            "2",
            "--height",
            "2",
            "--fill",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["shape"], "rect");
    assert!(json["output_size"].as_u64().unwrap() > 0);
}

#[test]
fn draw_dry_run() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("drawn.png");

    panimg()
        .args([
            "draw",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
            "--shape",
            "rect",
            "--width",
            "2",
            "--height",
            "2",
            "--dry-run",
        ])
        .assert()
        .success();

    assert!(!out_path.exists());
}

#[test]
fn draw_schema() {
    let output = panimg().args(["draw", "--schema"]).output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["command"], "draw");
}

#[test]
fn draw_missing_shape() {
    let dir = TempDir::new().unwrap();
    let img_path = create_test_png(dir.path(), "test.png");
    let out_path = dir.path().join("drawn.png");

    panimg()
        .args([
            "draw",
            img_path.to_str().unwrap(),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .failure();
}
