use super::common::{require_input, require_output, run_pipeline, PipelineInput};
use super::CommandResult;
use crate::app::{FlipArgs, RunContext};
use panimg_core::error::PanimgError;
use panimg_core::ops::flip::{FlipDirection, FlipOp};
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct FlipResult {
    input: String,
    output: String,
    direction: String,
    output_size: u64,
}

pub fn schema() -> pan_common::schema::CommandSchema {
    FlipOp::schema()
}

pub fn run(args: &FlipArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(
        &args.input,
        "panimg flip <input> -o <output> --direction horizontal",
    )?;
    let output = require_output(
        &args.output,
        &args.output_pos,
        "panimg flip <input> -o <output> --direction horizontal",
    )?;

    let direction_str = args
        .direction
        .as_deref()
        .ok_or_else(|| PanimgError::InvalidArgument {
            message: "missing required argument: --direction".into(),
            suggestion: "usage: panimg flip <input> -o <output> --direction horizontal".into(),
        })?;
    let direction = FlipDirection::parse(direction_str)?;

    let flip_op = FlipOp::new(direction);
    let pipeline = Pipeline::new().push(flip_op);
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
        &format!("Flipped {} → {} ({})", input, output, direction_str),
        &FlipResult {
            input: input.to_string(),
            output,
            direction: match direction {
                FlipDirection::Horizontal => "horizontal",
                FlipDirection::Vertical => "vertical",
            }
            .into(),
            output_size: out.output_size,
        },
    );
    Ok(0)
}
