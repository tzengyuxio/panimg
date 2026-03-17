use super::common::{require_input, require_output};
use super::CommandResult;
use crate::app::{GifSpeedArgs, RunContext};
use panimg_core::error::PanimgError;
use panimg_core::ops::animation;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct GifSpeedResult {
    input: String,
    output: String,
    speed_factor: f32,
    total_frames: usize,
    output_size: u64,
}

pub fn run(args: &GifSpeedArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(
        &args.input,
        "panimg gif-speed <input.gif> -o <output.gif> --speed 2.0",
    )?;
    let output_path_str = require_output(
        &args.output,
        &args.output_pos,
        "panimg gif-speed <input.gif> -o <output.gif> --speed 2.0",
    )?;

    let speed = args.speed.ok_or_else(|| PanimgError::InvalidArgument {
        message: "missing required argument: --speed".into(),
        suggestion: "use --speed 2.0 for 2x faster, or --speed 0.5 for half speed".into(),
    })?;

    let input_path = Path::new(input);

    if ctx.dry_run {
        let plan = serde_json::json!({
            "operation": "gif-speed",
            "input": input,
            "speed": speed,
        });
        ctx.print_output(&format!("Would change speed of {input} by {speed}x"), &plan);
        return Ok(0);
    }

    let (frames, _) = animation::extract_frames(input_path)?;
    let new_frames = animation::change_speed(&frames, speed)?;

    let output_path = Path::new(&output_path_str);
    animation::write_gif(&new_frames, output_path, true)?;

    let output_size = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);

    let result = GifSpeedResult {
        input: input.to_string(),
        output: output_path_str,
        speed_factor: speed,
        total_frames: new_frames.len(),
        output_size,
    };

    ctx.print_output(
        &format!(
            "Changed speed {}x: {} → {} ({} frames)",
            speed, result.input, result.output, result.total_frames
        ),
        &result,
    );

    Ok(0)
}
