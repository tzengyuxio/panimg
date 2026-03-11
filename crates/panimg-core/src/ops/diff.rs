use crate::error::{PanimgError, Result};
use image::{DynamicImage, RgbaImage};
use serde::Serialize;

/// Result of comparing two images.
#[derive(Debug, Serialize)]
pub struct DiffResult {
    /// Whether the images are identical.
    pub identical: bool,
    /// Number of pixels that differ beyond the threshold.
    pub diff_pixels: u64,
    /// Total number of pixels.
    pub total_pixels: u64,
    /// Percentage of pixels that differ (0.0-100.0).
    pub diff_percent: f64,
    /// Mean absolute error across all channels and pixels (0.0-255.0).
    pub mae: f64,
    /// Image dimensions match.
    pub dimensions_match: bool,
    /// Width of image A.
    pub width_a: u32,
    /// Height of image A.
    pub height_a: u32,
    /// Width of image B.
    pub width_b: u32,
    /// Height of image B.
    pub height_b: u32,
}

/// Compare two images and produce a diff visualization and statistics.
pub fn compare(
    img_a: &DynamicImage,
    img_b: &DynamicImage,
    threshold: u8,
) -> Result<(DiffResult, DynamicImage)> {
    let rgba_a = img_a.to_rgba8();
    let rgba_b = img_b.to_rgba8();

    let (wa, ha) = (rgba_a.width(), rgba_a.height());
    let (wb, hb) = (rgba_b.width(), rgba_b.height());

    let dimensions_match = wa == wb && ha == hb;

    // Use the larger dimensions for comparison canvas
    let w = wa.max(wb);
    let h = ha.max(hb);

    let total_pixels = w as u64 * h as u64;
    let mut diff_pixels = 0u64;
    let mut total_error = 0u64;
    let mut diff_img = RgbaImage::new(w, h);

    let thr = threshold as i32;

    for y in 0..h {
        for x in 0..w {
            let in_a = x < wa && y < ha;
            let in_b = x < wb && y < hb;

            match (in_a, in_b) {
                (true, true) => {
                    let pa = rgba_a.get_pixel(x, y);
                    let pb = rgba_b.get_pixel(x, y);

                    let dr = (pa[0] as i32 - pb[0] as i32).abs();
                    let dg = (pa[1] as i32 - pb[1] as i32).abs();
                    let db = (pa[2] as i32 - pb[2] as i32).abs();
                    let da = (pa[3] as i32 - pb[3] as i32).abs();

                    total_error += (dr + dg + db + da) as u64;

                    if dr > thr || dg > thr || db > thr || da > thr {
                        diff_pixels += 1;
                        // Highlight diff: red with intensity proportional to max channel diff
                        let max_diff = dr.max(dg).max(db).max(da);
                        let intensity =
                            ((max_diff as f32 / 255.0) * 255.0).clamp(50.0, 255.0) as u8;
                        diff_img.put_pixel(x, y, image::Rgba([255, 0, 0, intensity]));
                    } else {
                        // Same pixel: show faded grayscale version
                        let gray = ((pa[0] as u32 + pa[1] as u32 + pa[2] as u32) / 3) as u8;
                        let faded = gray / 3 + 40; // dim but visible
                        diff_img.put_pixel(x, y, image::Rgba([faded, faded, faded, 255]));
                    }
                }
                (true, false) => {
                    // Only in A: show as blue
                    diff_pixels += 1;
                    total_error += 255 * 4;
                    diff_img.put_pixel(x, y, image::Rgba([0, 0, 255, 200]));
                }
                (false, true) => {
                    // Only in B: show as green
                    diff_pixels += 1;
                    total_error += 255 * 4;
                    diff_img.put_pixel(x, y, image::Rgba([0, 255, 0, 200]));
                }
                (false, false) => {
                    // Neither (shouldn't happen, but just in case)
                    diff_img.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
                }
            }
        }
    }

    let mae = if total_pixels > 0 {
        total_error as f64 / (total_pixels as f64 * 4.0)
    } else {
        0.0
    };

    let diff_percent = if total_pixels > 0 {
        (diff_pixels as f64 / total_pixels as f64) * 100.0
    } else {
        0.0
    };

    let result = DiffResult {
        identical: diff_pixels == 0,
        diff_pixels,
        total_pixels,
        diff_percent,
        mae,
        dimensions_match,
        width_a: wa,
        height_a: ha,
        width_b: wb,
        height_b: hb,
    };

    Ok((result, DynamicImage::ImageRgba8(diff_img)))
}

/// Validate that comparison inputs are sensible.
pub fn validate_inputs(img_a: &DynamicImage, img_b: &DynamicImage) -> Result<()> {
    let (wa, ha) = (img_a.width(), img_a.height());
    let (wb, hb) = (img_b.width(), img_b.height());

    if wa == 0 || ha == 0 || wb == 0 || hb == 0 {
        return Err(PanimgError::InvalidArgument {
            message: "cannot compare zero-sized images".into(),
            suggestion: "provide valid image files".into(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn red_image(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            w,
            h,
            image::Rgba([255, 0, 0, 255]),
        ))
    }

    fn blue_image(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            w,
            h,
            image::Rgba([0, 0, 255, 255]),
        ))
    }

    #[test]
    fn identical_images() {
        let img = red_image(4, 4);
        let (result, _diff) = compare(&img, &img, 0).unwrap();
        assert!(result.identical);
        assert_eq!(result.diff_pixels, 0);
        assert_eq!(result.diff_percent, 0.0);
        assert!(result.dimensions_match);
    }

    #[test]
    fn completely_different() {
        let a = red_image(4, 4);
        let b = blue_image(4, 4);
        let (result, _diff) = compare(&a, &b, 0).unwrap();
        assert!(!result.identical);
        assert_eq!(result.diff_pixels, 16);
        assert_eq!(result.diff_percent, 100.0);
        assert!(result.dimensions_match);
    }

    #[test]
    fn threshold_hides_small_diffs() {
        let a = DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            4,
            4,
            image::Rgba([100, 100, 100, 255]),
        ));
        let b = DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            4,
            4,
            image::Rgba([105, 105, 105, 255]),
        ));

        // Threshold 10: diff of 5 per channel should be hidden
        let (result, _) = compare(&a, &b, 10).unwrap();
        assert!(result.identical);

        // Threshold 0: should detect the diff
        let (result, _) = compare(&a, &b, 0).unwrap();
        assert!(!result.identical);
    }

    #[test]
    fn different_dimensions() {
        let a = red_image(4, 4);
        let b = blue_image(6, 8);
        let (result, diff) = compare(&a, &b, 0).unwrap();
        assert!(!result.identical);
        assert!(!result.dimensions_match);
        assert_eq!(result.width_a, 4);
        assert_eq!(result.height_a, 4);
        assert_eq!(result.width_b, 6);
        assert_eq!(result.height_b, 8);
        // Diff image should be max dimensions
        assert_eq!(diff.width(), 6);
        assert_eq!(diff.height(), 8);
    }

    #[test]
    fn diff_visualization_size() {
        let a = red_image(8, 8);
        let b = blue_image(8, 8);
        let (_, diff) = compare(&a, &b, 0).unwrap();
        assert_eq!(diff.width(), 8);
        assert_eq!(diff.height(), 8);
    }

    #[test]
    fn mae_zero_for_identical() {
        let img = red_image(4, 4);
        let (result, _) = compare(&img, &img, 0).unwrap();
        assert_eq!(result.mae, 0.0);
    }

    #[test]
    fn mae_nonzero_for_different() {
        let a = red_image(4, 4);
        let b = blue_image(4, 4);
        let (result, _) = compare(&a, &b, 0).unwrap();
        assert!(result.mae > 0.0);
    }
}
