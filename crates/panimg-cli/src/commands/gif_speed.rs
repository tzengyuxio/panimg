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

pub fn run(args: &GifSpeedArgs, ctx: &RunContext) -> i32 {
    let input = match &args.input {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: input".into(),
                suggestion: "usage: panimg gif-speed <input.gif> -o <output.gif> --speed 2.0"
                    .into(),
            };
            return ctx.print_error(&err);
        }
    };

    let output_path_str = match args.output.as_ref().or(args.output_pos.as_ref()) {
        Some(o) => o.clone(),
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: output (-o)".into(),
                suggestion: "usage: panimg gif-speed <input.gif> -o <output.gif> --speed 2.0"
                    .into(),
            };
            return ctx.print_error(&err);
        }
    };

    let speed = match args.speed {
        Some(s) => s,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: --speed".into(),
                suggestion: "use --speed 2.0 for 2x faster, or --speed 0.5 for half speed".into(),
            };
            return ctx.print_error(&err);
        }
    };

    let input_path = Path::new(input);

    if ctx.dry_run {
        let plan = serde_json::json!({
            "operation": "gif-speed",
            "input": input,
            "speed": speed,
        });
        ctx.print_output(&format!("Would change speed of {input} by {speed}x"), &plan);
        return 0;
    }

    let (frames, _) = match animation::extract_frames(input_path) {
        Ok(r) => r,
        Err(e) => return ctx.print_error(&e),
    };

    let new_frames = match animation::change_speed(&frames, speed) {
        Ok(f) => f,
        Err(e) => return ctx.print_error(&e),
    };

    let output_path = Path::new(&output_path_str);
    if let Err(e) = animation::write_gif(&new_frames, output_path, true) {
        return ctx.print_error(&e);
    }

    let output_size = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);

    let result = GifSpeedResult {
        input: input.clone(),
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

    0
}
