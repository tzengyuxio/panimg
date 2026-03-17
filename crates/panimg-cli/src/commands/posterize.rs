use super::common::{require_input, require_output, run_pipeline, PipelineInput};
use super::CommandResult;
use crate::app::{PosterizeArgs, RunContext};
use panimg_core::ops::color::PosterizeOp;
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct PosterizeResult {
    input: String,
    output: String,
    levels: u8,
    width: u32,
    height: u32,
}

pub fn schema() -> pan_common::schema::CommandSchema {
    PosterizeOp::schema()
}

pub fn run(args: &PosterizeArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(
        &args.input,
        "panimg posterize <input> -o <output> --levels 4",
    )?;
    let output = require_output(
        &args.output,
        &args.output_pos,
        "panimg posterize <input> -o <output> --levels 4",
    )?;

    let levels = args.levels.unwrap_or(4);
    let op = PosterizeOp::new(levels)?;

    if ctx.dry_run {
        let desc = op.describe();
        ctx.print_output(
            &format!("Would posterize {input} to {levels} levels"),
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
        &format!("Posterized to {} levels: {} → {}", levels, input, output),
        &PosterizeResult {
            input: input.to_string(),
            output,
            levels,
            width: out.new_width,
            height: out.new_height,
        },
    );
    Ok(0)
}
