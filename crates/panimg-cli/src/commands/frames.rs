use crate::app::{FramesArgs, RunContext};
use panimg_core::error::PanimgError;
use panimg_core::ops::animation;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct FramesResult {
    input: String,
    total_frames: usize,
    output_dir: String,
    frames: Vec<FrameOutput>,
}

#[derive(Serialize)]
struct FrameOutput {
    index: usize,
    path: String,
    delay_ms: u32,
}

pub fn run(args: &FramesArgs, ctx: &RunContext) -> i32 {
    let input = match &args.input {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: input".into(),
                suggestion: "usage: panimg frames <input.gif> --output-dir ./frames".into(),
            };
            return ctx.print_error(&err);
        }
    };

    let input_path = Path::new(input);

    if ctx.dry_run {
        let plan = serde_json::json!({
            "operation": "frames",
            "input": input,
        });
        ctx.print_output(&format!("Would extract frames from {input}"), &plan);
        return 0;
    }

    let (frames, extract_result) = match animation::extract_frames(input_path) {
        Ok(r) => r,
        Err(e) => return ctx.print_error(&e),
    };

    let output_dir = args.output_dir.as_deref().unwrap_or(".");
    let out_dir = Path::new(output_dir);

    if !out_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(out_dir) {
            let err = PanimgError::IoError {
                message: e.to_string(),
                path: Some(out_dir.to_path_buf()),
                suggestion: "check the output directory path".into(),
            };
            return ctx.print_error(&err);
        }
    }

    let ext = args.frame_format.as_deref().unwrap_or("png");
    let prefix = args.prefix.as_deref().unwrap_or("frame");

    let mut frame_outputs = Vec::new();
    for (i, frame) in frames.iter().enumerate() {
        let filename = format!("{prefix}_{i:04}.{ext}");
        let frame_path = out_dir.join(&filename);

        if let Err(e) = animation::save_frame(frame, &frame_path) {
            return ctx.print_error(&e);
        }

        let delay_ms = extract_result
            .frames
            .get(i)
            .map(|f| f.delay_ms)
            .unwrap_or(0);

        frame_outputs.push(FrameOutput {
            index: i,
            path: frame_path.to_string_lossy().to_string(),
            delay_ms,
        });
    }

    let result = FramesResult {
        input: input.clone(),
        total_frames: frames.len(),
        output_dir: output_dir.to_string(),
        frames: frame_outputs,
    };

    ctx.print_output(
        &format!(
            "Extracted {} frames from {} → {}",
            result.total_frames, result.input, result.output_dir
        ),
        &result,
    );

    0
}
