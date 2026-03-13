#[cfg(feature = "tiny")]
mod compress_tests {
    use panimg_core::compress::{compress, CompressOptions};
    use std::path::Path;

    fn test_image_dir() -> &'static Path {
        // Use benches/images/kodak if available, otherwise create test images
        let kodak = Path::new("benches/images/kodak");
        if kodak.exists() {
            kodak
        } else {
            Path::new("tests/fixtures")
        }
    }

    fn create_test_png(path: &Path) {
        let img = image::RgbaImage::from_fn(100, 100, |x, y| {
            if (x + y) % 2 == 0 {
                image::Rgba([255, 0, 0, 255])
            } else {
                image::Rgba([0, 0, 255, 255])
            }
        });
        img.save(path).unwrap();
    }

    fn create_test_jpeg(path: &Path) {
        let img = image::RgbImage::from_fn(100, 100, |x, y| {
            let r = ((x * 255) / 100) as u8;
            let g = ((y * 255) / 100) as u8;
            image::Rgb([r, g, 128])
        });
        img.save(path).unwrap();
    }

    #[test]
    fn test_png_quantize_compress() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("test.png");
        let output = dir.path().join("test_tiny.png");

        create_test_png(&input);

        let options = CompressOptions {
            quality: None,
            max_colors: 256,
            lossless: false,
            strip_metadata: false,
        };

        let result = compress(&input, &output, &options).unwrap();
        assert!(output.exists());
        assert!(result.output_size > 0);
        assert!(result.output_size <= result.input_size);
    }

    #[test]
    fn test_png_lossless_compress() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("test.png");
        let output = dir.path().join("test_tiny.png");

        create_test_png(&input);

        let options = CompressOptions {
            quality: None,
            max_colors: 256,
            lossless: true,
            strip_metadata: false,
        };

        let result = compress(&input, &output, &options).unwrap();
        assert!(output.exists());
        assert!(result.output_size > 0);
    }

    #[test]
    fn test_png_max_colors() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("test.png");
        let output = dir.path().join("test_tiny.png");

        create_test_png(&input);

        let options = CompressOptions {
            quality: None,
            max_colors: 16,
            lossless: false,
            strip_metadata: false,
        };

        let result = compress(&input, &output, &options).unwrap();
        assert!(output.exists());
        assert!(result.output_size > 0);
    }

    #[test]
    fn test_jpeg_compress() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("test.jpg");
        let output = dir.path().join("test_tiny.jpg");

        create_test_jpeg(&input);

        let options = CompressOptions {
            quality: Some(60),
            max_colors: 256,
            lossless: false,
            strip_metadata: false,
        };

        let result = compress(&input, &output, &options).unwrap();
        assert!(output.exists());
        assert!(result.output_size > 0);
    }

    #[test]
    fn test_jpeg_default_quality() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("test.jpg");
        let output = dir.path().join("test_tiny.jpg");

        create_test_jpeg(&input);

        let options = CompressOptions::default();

        let result = compress(&input, &output, &options).unwrap();
        assert!(output.exists());
        assert!(result.output_size > 0);
    }

    #[test]
    fn test_unsupported_format() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("test.bmp");
        let output = dir.path().join("test_tiny.bmp");

        // Create a BMP
        let img = image::RgbImage::from_fn(10, 10, |_, _| image::Rgb([128, 128, 128]));
        img.save(&input).unwrap();

        let options = CompressOptions::default();
        let result = compress(&input, &output, &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_strip_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("test.png");
        let output = dir.path().join("test_tiny.png");

        create_test_png(&input);

        let options = CompressOptions {
            quality: None,
            max_colors: 256,
            lossless: false,
            strip_metadata: true,
        };

        let result = compress(&input, &output, &options).unwrap();
        assert!(output.exists());
        assert!(result.output_size > 0);
    }

    #[test]
    fn test_compress_result_savings() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("test.png");
        let output = dir.path().join("test_tiny.png");

        // Create a larger image with gradients for more realistic test
        let img = image::RgbaImage::from_fn(200, 200, |x, y| {
            image::Rgba([(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8, 255])
        });
        img.save(&input).unwrap();

        let options = CompressOptions {
            quality: None,
            max_colors: 128,
            lossless: false,
            strip_metadata: true,
        };

        let result = compress(&input, &output, &options).unwrap();
        // With quantization to 128 colors, we should see some savings
        assert!(result.savings_percent > 0.0 || result.output_size <= result.input_size);
    }

    #[test]
    fn test_kodak_png_if_available() {
        let kodak_dir = test_image_dir();
        let input = kodak_dir.join("kodim01.png");
        if !input.exists() {
            // Skip if test images not available
            return;
        }

        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("kodim01_tiny.png");

        let options = CompressOptions {
            quality: None,
            max_colors: 256,
            lossless: false,
            strip_metadata: true,
        };

        let result = compress(&input, &output, &options).unwrap();
        assert!(output.exists());
        assert!(result.output_size > 0);
        println!(
            "kodim01.png: {} → {} ({:.1}% savings)",
            result.input_size, result.output_size, result.savings_percent
        );
    }
}
