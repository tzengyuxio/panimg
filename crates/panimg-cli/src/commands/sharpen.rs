use super::common::{require_input, require_output, run_pipeline, PipelineInput};
use super::CommandResult;
use crate::app::{RunContext, SharpenArgs};
use panimg_core::error::PanimgError;
use panimg_core::ops::sharpen::SharpenOp;
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct SharpenResult {
    input: String,
    output: String,
    sigma: f32,
    threshold: i32,
    output_size: u64,
}

pub fn schema() -> pan_common::schema::CommandSchema {
    SharpenOp::schema()
}

pub fn run(args: &SharpenArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(
        &args.input,
        "panimg sharpen <input> -o <output> --sigma 1.0",
    )?;
    let output = require_output(
        &args.output,
        &args.output_pos,
        "panimg sharpen <input> -o <output> --sigma 1.0",
    )?;

    let sigma = args.sigma.ok_or_else(|| PanimgError::InvalidArgument {
        message: "missing required argument: --sigma".into(),
        suggestion: "usage: panimg sharpen <input> -o <output> --sigma 1.0".into(),
    })?;
    let threshold = args.threshold.unwrap_or(0);

    let sharpen_op = SharpenOp::new(sigma, threshold)?;
    let pipeline = Pipeline::new().push(sharpen_op);
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
            "Sharpened {} → {} (sigma={}, threshold={})",
            input, output, sigma, threshold
        ),
        &SharpenResult {
            input: input.to_string(),
            output,
            sigma,
            threshold,
            output_size: out.output_size,
        },
    );
    Ok(0)
}
