use super::common::{require_input, require_output, run_pipeline, PipelineInput};
use super::CommandResult;
use crate::app::{RotateArgs, RunContext};
use panimg_core::color::parse_color;
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::ops::rotate::{RotateAngle, RotateOp};
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct RotateResult {
    input: String,
    output: String,
    angle: f64,
    new_width: u32,
    new_height: u32,
    output_size: u64,
}

/// Determine the default background color string based on the output format.
/// - Formats that support alpha (PNG, WebP, TIFF, GIF) → transparent
/// - Opaque formats (JPEG, BMP) → white
fn default_background_str(format: &ImageFormat) -> &'static str {
    match format {
        ImageFormat::Jpeg | ImageFormat::Bmp => "white",
        _ => "transparent",
    }
}

pub fn schema() -> pan_common::schema::CommandSchema {
    RotateOp::schema()
}

pub fn run(args: &RotateArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(&args.input, "panimg rotate <input> -o <output> --angle 90")?;
    let output = require_output(
        &args.output,
        &args.output_pos,
        "panimg rotate <input> -o <output> --angle 90",
    )?;

    let angle_str = args
        .angle
        .as_deref()
        .ok_or_else(|| PanimgError::InvalidArgument {
            message: "missing required argument: --angle".into(),
            suggestion: "usage: panimg rotate <input> -o <output> --angle 90".into(),
        })?;
    let angle = RotateAngle::parse(angle_str)?;

    let input_path = Path::new(input);
    let output_path = Path::new(&output);

    // Determine output format for default background color
    let out_format = ImageFormat::from_path_extension(output_path)
        .or_else(|| ImageFormat::from_path(input_path))
        .unwrap_or(ImageFormat::Png);

    // Parse background color
    let bg_str = args
        .background
        .as_deref()
        .unwrap_or_else(|| default_background_str(&out_format));
    let background = parse_color(bg_str)?;

    let rotate_op = RotateOp::new(angle).with_background(background);
    let pipeline = Pipeline::new().push(rotate_op);
    let pi = PipelineInput {
        input_path,
        output_path,
        quality: args.quality,
        strip_metadata: args.strip,
    };

    let Some(out) = run_pipeline(&pipeline, &pi, ctx)? else {
        return Ok(0);
    };

    ctx.print_output(
        &format!(
            "Rotated {} → {} ({}°, {}x{})",
            input,
            output,
            angle.degrees_f64(),
            out.new_width,
            out.new_height
        ),
        &RotateResult {
            input: input.to_string(),
            output,
            angle: angle.degrees_f64(),
            new_width: out.new_width,
            new_height: out.new_height,
            output_size: out.output_size,
        },
    );
    Ok(0)
}
