use super::common::{require_input, require_output, run_pipeline, PipelineInput};
use super::CommandResult;
use crate::app::{GrayscaleArgs, RunContext};
use panimg_core::ops::grayscale::GrayscaleOp;
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct GrayscaleResult {
    input: String,
    output: String,
    output_size: u64,
}

pub fn schema() -> pan_common::schema::CommandSchema {
    GrayscaleOp::schema()
}

pub fn run(args: &GrayscaleArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(&args.input, "panimg grayscale <input> -o <output>")?;
    let output = require_output(
        &args.output,
        &args.output_pos,
        "panimg grayscale <input> -o <output>",
    )?;

    let pipeline = Pipeline::new().push(GrayscaleOp::new());
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
        &format!("Grayscale {} → {}", input, output),
        &GrayscaleResult {
            input: input.to_string(),
            output,
            output_size: out.output_size,
        },
    );
    Ok(0)
}
