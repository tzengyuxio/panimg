use super::common::{require_input, require_output, run_pipeline, PipelineInput};
use super::CommandResult;
use crate::app::{AutoOrientArgs, RunContext};
use panimg_core::ops::orient::AutoOrientOp;
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct AutoOrientResult {
    input: String,
    output: String,
    new_width: u32,
    new_height: u32,
    output_size: u64,
}

pub fn schema() -> pan_common::schema::CommandSchema {
    AutoOrientOp::schema()
}

pub fn run(args: &AutoOrientArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(&args.input, "panimg auto-orient <input> -o <output>")?;
    let output = require_output(
        &args.output,
        &args.output_pos,
        "panimg auto-orient <input> -o <output>",
    )?;

    let input_path = Path::new(input);
    let orient_op = AutoOrientOp::from_path(input_path);
    let pipeline = Pipeline::new().push(orient_op);
    let pi = PipelineInput {
        input_path,
        output_path: Path::new(&output),
        quality: args.quality,
        strip_metadata: args.strip,
    };

    let Some(out) = run_pipeline(&pipeline, &pi, ctx)? else {
        return Ok(0);
    };

    ctx.print_output(
        &format!(
            "Auto-oriented {} → {} ({}x{})",
            input, output, out.new_width, out.new_height
        ),
        &AutoOrientResult {
            input: input.to_string(),
            output,
            new_width: out.new_width,
            new_height: out.new_height,
            output_size: out.output_size,
        },
    );
    Ok(0)
}
