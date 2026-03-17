use super::common::{require_input, require_output, run_pipeline, PipelineInput};
use super::CommandResult;
use crate::app::{ContrastArgs, RunContext};
use panimg_core::error::PanimgError;
use panimg_core::ops::contrast::ContrastOp;
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct ContrastResult {
    input: String,
    output: String,
    value: f32,
    output_size: u64,
}

pub fn schema() -> pan_common::schema::CommandSchema {
    ContrastOp::schema()
}

pub fn run(args: &ContrastArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(
        &args.input,
        "panimg contrast <input> -o <output> --value 30",
    )?;
    let output = require_output(
        &args.output,
        &args.output_pos,
        "panimg contrast <input> -o <output> --value 30",
    )?;

    let value = args.value.ok_or_else(|| PanimgError::InvalidArgument {
        message: "missing required argument: --value".into(),
        suggestion: "usage: panimg contrast <input> -o <output> --value 30".into(),
    })?;

    let contrast_op = ContrastOp::new(value)?;
    let pipeline = Pipeline::new().push(contrast_op);
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
        &format!("Contrast {} → {} ({})", input, output, value),
        &ContrastResult {
            input: input.to_string(),
            output,
            value,
            output_size: out.output_size,
        },
    );
    Ok(0)
}
