use super::common::{require_input, require_output, run_pipeline, PipelineInput};
use super::CommandResult;
use crate::app::{CropArgs, RunContext};
use panimg_core::error::PanimgError;
use panimg_core::ops::crop::CropOp;
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct CropResult {
    input: String,
    output: String,
    x: u32,
    y: u32,
    crop_width: u32,
    crop_height: u32,
    output_size: u64,
}

pub fn schema() -> pan_common::schema::CommandSchema {
    CropOp::schema()
}

pub fn run(args: &CropArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(
        &args.input,
        "panimg crop <input> -o <output> --x 0 --y 0 --width 100 --height 100",
    )?;
    let output = require_output(
        &args.output,
        &args.output_pos,
        "panimg crop <input> -o <output> --width 100 --height 100",
    )?;

    let width = args.width.ok_or_else(|| PanimgError::InvalidArgument {
        message: "missing required argument: --width".into(),
        suggestion: "specify crop dimensions with --width and --height".into(),
    })?;
    let height = args.height.ok_or_else(|| PanimgError::InvalidArgument {
        message: "missing required argument: --height".into(),
        suggestion: "specify crop dimensions with --width and --height".into(),
    })?;

    let crop_op = CropOp::new(args.x, args.y, width, height)?;
    let pipeline = Pipeline::new().push(crop_op);
    let pi = PipelineInput {
        input_path: Path::new(input),
        output_path: Path::new(&output),
        quality: args.quality,
        strip_metadata: args.strip,
    };

    let Some(out) = run_pipeline(&pipeline, &pi, ctx)? else {
        return Ok(0);
    };

    ctx.print_output(
        &format!(
            "Cropped {} → {} ({}x{} at {}, {})",
            input, output, width, height, args.x, args.y
        ),
        &CropResult {
            input: input.to_string(),
            output,
            x: args.x,
            y: args.y,
            crop_width: width,
            crop_height: height,
            output_size: out.output_size,
        },
    );
    Ok(0)
}
