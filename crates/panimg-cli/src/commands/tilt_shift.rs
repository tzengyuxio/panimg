use super::common::{require_input, require_output, run_pipeline, PipelineInput};
use super::CommandResult;
use crate::app::{RunContext, TiltShiftArgs};
use panimg_core::ops::tilt_shift::TiltShiftOp;
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct TiltShiftResult {
    input: String,
    output: String,
    sigma: f32,
    focus_position: f32,
    focus_width: f32,
    transition: f32,
    saturation: f32,
    output_size: u64,
}

pub fn schema() -> pan_common::schema::CommandSchema {
    TiltShiftOp::schema()
}

pub fn run(args: &TiltShiftArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(&args.input, "panimg tilt-shift <input> -o <output>")?;
    let output = require_output(
        &args.output,
        &args.output_pos,
        "panimg tilt-shift <input> -o <output>",
    )?;

    let sigma = args.sigma.unwrap_or(8.0);
    let focus_position = args.focus_position.unwrap_or(0.5);
    let focus_width = args.focus_width.unwrap_or(0.15);
    let transition = args.transition.unwrap_or(0.2);
    let saturation = args.saturation.unwrap_or(1.0);

    let op = TiltShiftOp::new(sigma, focus_position, focus_width, transition, saturation)?;
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
        &format!(
            "Tilt-shift {} → {} (sigma={}, saturation={})",
            input, output, sigma, saturation
        ),
        &TiltShiftResult {
            input: input.to_string(),
            output,
            sigma,
            focus_position,
            focus_width,
            transition,
            saturation,
            output_size: out.output_size,
        },
    );
    Ok(0)
}
