use super::common::require_output;
use super::CommandResult;
use crate::app::{AnimateArgs, RunContext};
use panimg_core::codec::CodecRegistry;
use panimg_core::error::PanimgError;
use panimg_core::ops::animation;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct AnimateResult {
    output: String,
    total_frames: usize,
    delay_ms: u32,
    output_size: u64,
}

pub fn run(args: &AnimateArgs, ctx: &RunContext) -> CommandResult {
    let pattern = args
        .pattern
        .as_deref()
        .ok_or_else(|| PanimgError::InvalidArgument {
            message: "missing required argument: pattern".into(),
            suggestion: "usage: panimg animate <pattern> -o <output.gif> --delay 100".into(),
        })?;

    let output_path_str = require_output(
        &args.output,
        &args.output_pos,
        "panimg animate 'frames/*.png' -o output.gif --delay 100",
    )?;

    // Expand glob pattern and sort
    let mut files: Vec<std::path::PathBuf> = match glob::glob(pattern) {
        Ok(paths) => paths
            .filter_map(|p| p.ok())
            .filter(|p| p.is_file())
            .collect(),
        Err(e) => {
            return Err(PanimgError::InvalidArgument {
                message: format!("invalid glob pattern: {e}"),
                suggestion: "use a pattern like 'frames/*.png'".into(),
            });
        }
    };
    files.sort();

    if files.is_empty() {
        return Err(PanimgError::InvalidArgument {
            message: format!("no files matched pattern: '{pattern}'"),
            suggestion: "check the glob pattern and ensure matching files exist".into(),
        });
    }

    let delay_ms = args.delay.unwrap_or(100);
    let repeat = !args.no_repeat;

    if ctx.dry_run {
        let plan = serde_json::json!({
            "operation": "animate",
            "pattern": pattern,
            "total_frames": files.len(),
            "delay_ms": delay_ms,
            "repeat": repeat,
        });
        ctx.print_output(
            &format!(
                "Would assemble {} frames into {} (delay={}ms)",
                files.len(),
                output_path_str,
                delay_ms
            ),
            &plan,
        );
        return Ok(0);
    }

    // Load all images
    let decode_opts = ctx.decode_options();
    let mut images = Vec::with_capacity(files.len());
    for file in &files {
        let img = CodecRegistry::decode_with_options(file, &decode_opts)?;
        images.push(img);
    }

    let output_path = Path::new(&output_path_str);
    animation::assemble_gif(&images, output_path, delay_ms, repeat)?;

    let output_size = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);

    let result = AnimateResult {
        output: output_path_str,
        total_frames: images.len(),
        delay_ms,
        output_size,
    };

    ctx.print_output(
        &format!(
            "Assembled {} frames → {} (delay={}ms, {})",
            result.total_frames,
            result.output,
            result.delay_ms,
            if repeat { "loop" } else { "no-loop" }
        ),
        &result,
    );

    Ok(0)
}
