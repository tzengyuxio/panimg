use super::common::{require_input, require_output, run_pipeline, PipelineInput};
use super::CommandResult;
use crate::app::{RunContext, TrimArgs};
use panimg_core::ops::trim::TrimOp;
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct TrimResult {
    input: String,
    output: String,
    tolerance: u8,
    original_width: u32,
    original_height: u32,
    trimmed_width: u32,
    trimmed_height: u32,
    output_size: u64,
}

pub fn schema() -> pan_common::schema::CommandSchema {
    TrimOp::schema()
}

pub fn run(args: &TrimArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(&args.input, "panimg trim <input> -o <output>")?;
    let output = require_output(
        &args.output,
        &args.output_pos,
        "panimg trim <input> -o <output>",
    )?;

    let tolerance = args.tolerance.unwrap_or(10);
    let trim_op = TrimOp::new(tolerance)?;
    let pipeline = Pipeline::new().push(trim_op);
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
            "Trimmed {} → {} ({}x{} → {}x{})",
            input, output, out.original_width, out.original_height, out.new_width, out.new_height
        ),
        &TrimResult {
            input: input.to_string(),
            output,
            tolerance,
            original_width: out.original_width,
            original_height: out.original_height,
            trimmed_width: out.new_width,
            trimmed_height: out.new_height,
            output_size: out.output_size,
        },
    );
    Ok(0)
}
