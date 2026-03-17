use super::common::{require_input, require_output, run_pipeline, PipelineInput};
use super::CommandResult;
use crate::app::{HueRotateArgs, RunContext};
use panimg_core::error::PanimgError;
use panimg_core::ops::hue_rotate::HueRotateOp;
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct HueRotateResult {
    input: String,
    output: String,
    degrees: i32,
    output_size: u64,
}

pub fn schema() -> pan_common::schema::CommandSchema {
    HueRotateOp::schema()
}

pub fn run(args: &HueRotateArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(
        &args.input,
        "panimg hue-rotate <input> -o <output> --degrees 90",
    )?;
    let output = require_output(
        &args.output,
        &args.output_pos,
        "panimg hue-rotate <input> -o <output> --degrees 90",
    )?;

    let degrees = args.degrees.ok_or_else(|| PanimgError::InvalidArgument {
        message: "missing required argument: --degrees".into(),
        suggestion: "usage: panimg hue-rotate <input> -o <output> --degrees 90".into(),
    })?;

    let hue_op = HueRotateOp::new(degrees)?;
    let pipeline = Pipeline::new().push(hue_op);
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
        &format!("Hue rotated {} → {} ({}°)", input, output, degrees),
        &HueRotateResult {
            input: input.to_string(),
            output,
            degrees,
            output_size: out.output_size,
        },
    );
    Ok(0)
}
