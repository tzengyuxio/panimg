use super::common::{require_input, require_output, run_pipeline, PipelineInput};
use super::CommandResult;
use crate::app::{RunContext, SaturateArgs};
use panimg_core::error::PanimgError;
use panimg_core::ops::color::SaturateOp;
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct SaturateResult {
    input: String,
    output: String,
    factor: f32,
    width: u32,
    height: u32,
}

pub fn schema() -> pan_common::schema::CommandSchema {
    SaturateOp::schema()
}

pub fn run(args: &SaturateArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(
        &args.input,
        "panimg saturate <input> -o <output> --factor 1.5",
    )?;
    let output = require_output(
        &args.output,
        &args.output_pos,
        "panimg saturate <input> -o <output> --factor 1.5",
    )?;

    let factor = args.factor.ok_or_else(|| PanimgError::InvalidArgument {
        message: "missing required argument: --factor".into(),
        suggestion: "use --factor 1.5 to increase saturation or --factor 0.5 to decrease".into(),
    })?;

    let op = SaturateOp::new(factor)?;

    if ctx.dry_run {
        let desc = op.describe();
        ctx.print_output(
            &format!("Would adjust saturation of {input} by factor {factor}"),
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
        &format!("Saturation adjusted {}x: {} → {}", factor, input, output),
        &SaturateResult {
            input: input.to_string(),
            output,
            factor,
            width: out.new_width,
            height: out.new_height,
        },
    );
    Ok(0)
}
