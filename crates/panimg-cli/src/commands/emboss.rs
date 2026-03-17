use super::common::{require_input, require_output, run_pipeline, PipelineInput};
use super::CommandResult;
use crate::app::{EmbossArgs, RunContext};
use panimg_core::ops::emboss::EmbossOp;
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct EmbossResult {
    input: String,
    output: String,
    output_size: u64,
}

pub fn schema() -> pan_common::schema::CommandSchema {
    EmbossOp::schema()
}

pub fn run(args: &EmbossArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(&args.input, "panimg emboss <input> -o <output>")?;
    let output = require_output(
        &args.output,
        &args.output_pos,
        "panimg emboss <input> -o <output>",
    )?;

    let pipeline = Pipeline::new().push(EmbossOp::new());
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
        &format!("Emboss {} → {}", input, output),
        &EmbossResult {
            input: input.to_string(),
            output,
            output_size: out.output_size,
        },
    );
    Ok(0)
}
