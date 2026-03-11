use crate::error::{PanimgError, Result};
use image::codecs::gif::{GifDecoder, GifEncoder, Repeat};
#[cfg(test)]
use image::RgbaImage;
use image::{AnimationDecoder, DynamicImage, Frame};
use serde::Serialize;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Information about a single frame in an animation.
#[derive(Debug, Serialize)]
pub struct FrameInfo {
    pub index: usize,
    pub width: u32,
    pub height: u32,
    pub delay_ms: u32,
}

/// Result of extracting frames.
#[derive(Debug, Serialize)]
pub struct ExtractResult {
    pub total_frames: usize,
    pub frames: Vec<FrameInfo>,
}

/// Extract all frames from an animated GIF.
pub fn extract_frames(path: &Path) -> Result<(Vec<Frame>, ExtractResult)> {
    let file = File::open(path).map_err(|e| PanimgError::IoError {
        message: e.to_string(),
        path: Some(path.to_path_buf()),
        suggestion: "check that the file exists and is readable".into(),
    })?;

    let decoder = GifDecoder::new(BufReader::new(file)).map_err(|e| PanimgError::DecodeError {
        message: format!("failed to decode GIF: {e}"),
        path: Some(path.to_path_buf()),
        suggestion: "ensure the file is a valid animated GIF".into(),
    })?;

    let frames: Vec<Frame> =
        decoder
            .into_frames()
            .collect_frames()
            .map_err(|e| PanimgError::DecodeError {
                message: format!("failed to read GIF frames: {e}"),
                path: Some(path.to_path_buf()),
                suggestion: "the GIF file may be corrupted".into(),
            })?;

    if frames.is_empty() {
        return Err(PanimgError::InvalidArgument {
            message: "GIF has no frames".into(),
            suggestion: "provide an animated GIF with at least one frame".into(),
        });
    }

    let frame_infos: Vec<FrameInfo> = frames
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let (numer, denom) = f.delay().numer_denom_ms();
            let delay_ms = if denom == 0 { 0 } else { numer / denom };
            FrameInfo {
                index: i,
                width: f.buffer().width(),
                height: f.buffer().height(),
                delay_ms,
            }
        })
        .collect();

    let result = ExtractResult {
        total_frames: frames.len(),
        frames: frame_infos,
    };

    Ok((frames, result))
}

/// Save a single frame as an image.
pub fn save_frame(frame: &Frame, output_path: &Path) -> Result<()> {
    let img = DynamicImage::ImageRgba8(frame.buffer().clone());
    img.save(output_path).map_err(|e| PanimgError::EncodeError {
        message: format!("failed to save frame: {e}"),
        path: Some(output_path.to_path_buf()),
        suggestion: "check the output path and format".into(),
    })
}

/// Assemble multiple images into an animated GIF.
pub fn assemble_gif(
    images: &[DynamicImage],
    output_path: &Path,
    delay_ms: u32,
    repeat: bool,
) -> Result<()> {
    if images.is_empty() {
        return Err(PanimgError::InvalidArgument {
            message: "no images to assemble".into(),
            suggestion: "provide at least one image".into(),
        });
    }

    let file = File::create(output_path).map_err(|e| PanimgError::IoError {
        message: e.to_string(),
        path: Some(output_path.to_path_buf()),
        suggestion: "check the output path is writable".into(),
    })?;

    let mut encoder = GifEncoder::new(file);
    if repeat {
        encoder
            .set_repeat(Repeat::Infinite)
            .map_err(|e| PanimgError::EncodeError {
                message: format!("failed to set repeat: {e}"),
                path: Some(output_path.to_path_buf()),
                suggestion: "".into(),
            })?;
    }

    for img in images {
        let rgba = img.to_rgba8();
        let frame = Frame::from_parts(rgba, 0, 0, image::Delay::from_numer_denom_ms(delay_ms, 1));
        encoder
            .encode_frame(frame)
            .map_err(|e| PanimgError::EncodeError {
                message: format!("failed to encode frame: {e}"),
                path: Some(output_path.to_path_buf()),
                suggestion: "".into(),
            })?;
    }

    Ok(())
}

/// Change the speed of an animated GIF by adjusting frame delays.
pub fn change_speed(frames: &[Frame], speed_factor: f32) -> Result<Vec<Frame>> {
    if speed_factor <= 0.0 {
        return Err(PanimgError::InvalidArgument {
            message: format!("speed factor must be positive, got {speed_factor}"),
            suggestion: "use a value like 2.0 for 2x speed or 0.5 for half speed".into(),
        });
    }

    Ok(frames
        .iter()
        .map(|f| {
            let (numer, denom) = f.delay().numer_denom_ms();
            let delay_ms = if denom == 0 { 100 } else { numer / denom };
            let new_delay = ((delay_ms as f32) / speed_factor).round().max(10.0) as u32;
            Frame::from_parts(
                f.buffer().clone(),
                0,
                0,
                image::Delay::from_numer_denom_ms(new_delay, 1),
            )
        })
        .collect())
}

/// Write frames to a GIF file.
pub fn write_gif(frames: &[Frame], output_path: &Path, repeat: bool) -> Result<()> {
    let file = File::create(output_path).map_err(|e| PanimgError::IoError {
        message: e.to_string(),
        path: Some(output_path.to_path_buf()),
        suggestion: "check the output path is writable".into(),
    })?;

    let mut encoder = GifEncoder::new(file);
    if repeat {
        encoder
            .set_repeat(Repeat::Infinite)
            .map_err(|e| PanimgError::EncodeError {
                message: format!("failed to set repeat: {e}"),
                path: Some(output_path.to_path_buf()),
                suggestion: "".into(),
            })?;
    }

    for frame in frames {
        let new_frame = Frame::from_parts(frame.buffer().clone(), 0, 0, frame.delay());
        encoder
            .encode_frame(new_frame)
            .map_err(|e| PanimgError::EncodeError {
                message: format!("failed to encode frame: {e}"),
                path: Some(output_path.to_path_buf()),
                suggestion: "".into(),
            })?;
    }

    Ok(())
}

/// Create a simple test animated GIF for testing.
#[cfg(test)]
pub fn create_test_gif(path: &Path, frame_count: usize) {
    let mut images = Vec::new();
    for i in 0..frame_count {
        let val = ((i as f32 / frame_count as f32) * 255.0) as u8;
        let img =
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(4, 4, image::Rgba([val, 0, 0, 255])));
        images.push(img);
    }
    assemble_gif(&images, path, 100, true).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn create_and_extract_gif() {
        let dir = TempDir::new().unwrap();
        let gif_path = dir.path().join("test.gif");

        create_test_gif(&gif_path, 3);
        assert!(gif_path.exists());

        let (frames, result) = extract_frames(&gif_path).unwrap();
        assert_eq!(result.total_frames, 3);
        assert_eq!(frames.len(), 3);
    }

    #[test]
    fn save_individual_frame() {
        let dir = TempDir::new().unwrap();
        let gif_path = dir.path().join("test.gif");
        create_test_gif(&gif_path, 2);

        let (frames, _) = extract_frames(&gif_path).unwrap();
        let frame_path = dir.path().join("frame0.png");
        save_frame(&frames[0], &frame_path).unwrap();
        assert!(frame_path.exists());

        let img = image::open(&frame_path).unwrap();
        assert_eq!(img.width(), 4);
        assert_eq!(img.height(), 4);
    }

    #[test]
    fn change_speed_doubles() {
        let dir = TempDir::new().unwrap();
        let gif_path = dir.path().join("test.gif");
        create_test_gif(&gif_path, 3);

        let (frames, _) = extract_frames(&gif_path).unwrap();
        let fast_frames = change_speed(&frames, 2.0).unwrap();
        assert_eq!(fast_frames.len(), 3);

        // Delay should be halved (100ms -> 50ms)
        let (numer, denom) = fast_frames[0].delay().numer_denom_ms();
        let delay = if denom == 0 { 0 } else { numer / denom };
        assert_eq!(delay, 50);
    }

    #[test]
    fn change_speed_invalid() {
        let dir = TempDir::new().unwrap();
        let gif_path = dir.path().join("test.gif");
        create_test_gif(&gif_path, 2);

        let (frames, _) = extract_frames(&gif_path).unwrap();
        assert!(change_speed(&frames, 0.0).is_err());
        assert!(change_speed(&frames, -1.0).is_err());
    }

    #[test]
    fn assemble_empty_error() {
        let dir = TempDir::new().unwrap();
        let out = dir.path().join("out.gif");
        assert!(assemble_gif(&[], &out, 100, true).is_err());
    }

    #[test]
    fn roundtrip_speed_change() {
        let dir = TempDir::new().unwrap();
        let gif_path = dir.path().join("test.gif");
        let out_path = dir.path().join("fast.gif");
        create_test_gif(&gif_path, 3);

        let (frames, _) = extract_frames(&gif_path).unwrap();
        let fast_frames = change_speed(&frames, 2.0).unwrap();
        write_gif(&fast_frames, &out_path, true).unwrap();

        assert!(out_path.exists());
        let (result_frames, _) = extract_frames(&out_path).unwrap();
        assert_eq!(result_frames.len(), 3);
    }
}
