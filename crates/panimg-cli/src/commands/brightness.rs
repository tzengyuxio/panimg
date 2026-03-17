use super::common::{require_input, require_output, run_pipeline, PipelineInput};
use super::CommandResult;
use crate::app::{BrightnessArgs, RunContext};
use panimg_core::error::PanimgError;
use panimg_core::ops::brightness::BrightnessOp;
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct BrightnessResult {
    input: String,
    output: String,
    value: i32,
    output_size: u64,
}

pub fn schema() -> pan_common::schema::CommandSchema {
    BrightnessOp::schema()
}

pub fn run(args: &BrightnessArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(
        &args.input,
        "panimg brightness <input> -o <output> --value 20",
    )?;
    let output = require_output(
        &args.output,
        &args.output_pos,
        "panimg brightness <input> -o <output> --value 20",
    )?;

    let value = args.value.ok_or_else(|| PanimgError::InvalidArgument {
        message: "missing required argument: --value".into(),
        suggestion: "usage: panimg brightness <input> -o <output> --value 20".into(),
    })?;

    let brightness_op = BrightnessOp::new(value)?;
    let pipeline = Pipeline::new().push(brightness_op);
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
        &format!("Brightness {} → {} ({})", input, output, value),
        &BrightnessResult {
            input: input.to_string(),
            output,
            value,
            output_size: out.output_size,
        },
    );
    Ok(0)
}
