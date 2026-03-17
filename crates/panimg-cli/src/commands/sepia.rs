use super::common::{require_input, require_output, run_pipeline, PipelineInput};
use super::CommandResult;
use crate::app::{RunContext, SepiaArgs};
use panimg_core::ops::color::SepiaOp;
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct SepiaResult {
    input: String,
    output: String,
    intensity: f32,
    width: u32,
    height: u32,
}

pub fn schema() -> pan_common::schema::CommandSchema {
    SepiaOp::schema()
}

pub fn run(args: &SepiaArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(&args.input, "panimg sepia <input> -o <output>")?;
    let output = require_output(
        &args.output,
        &args.output_pos,
        "panimg sepia <input> -o <output>",
    )?;

    let intensity = args.intensity.unwrap_or(1.0);
    let op = SepiaOp::new(intensity)?;

    if ctx.dry_run {
        let desc = op.describe();
        ctx.print_output(
            &format!("Would apply sepia tone to {input} at intensity {intensity}"),
            &desc,
        );
        return Ok(0);
    }

    let pipeline = Pipeline::new().push(op);
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
        &format!("Sepia applied: {} → {}", input, output),
        &SepiaResult {
            input: input.to_string(),
            output,
            intensity,
            width: out.new_width,
            height: out.new_height,
        },
    );
    Ok(0)
}
