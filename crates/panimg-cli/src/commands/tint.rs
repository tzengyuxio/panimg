use super::common::{require_input, require_output, run_pipeline, PipelineInput};
use super::CommandResult;
use crate::app::{RunContext, TintArgs};
use panimg_core::color::parse_color;
use panimg_core::error::PanimgError;
use panimg_core::ops::color::TintOp;
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct TintResult {
    input: String,
    output: String,
    color: String,
    strength: f32,
    width: u32,
    height: u32,
}

pub fn schema() -> pan_common::schema::CommandSchema {
    TintOp::schema()
}

pub fn run(args: &TintArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(
        &args.input,
        "panimg tint <input> -o <output> --color red --strength 0.5",
    )?;
    let output = require_output(
        &args.output,
        &args.output_pos,
        "panimg tint <input> -o <output> --color red",
    )?;

    let color_str = args
        .color
        .as_deref()
        .ok_or_else(|| PanimgError::InvalidArgument {
            message: "missing required argument: --color".into(),
            suggestion: "use --color red, --color '#FF0000', or --color '255,0,0'".into(),
        })?;
    let rgba = parse_color(color_str)?;
    let strength = args.strength.unwrap_or(0.5);

    let op = TintOp::new(rgba[0], rgba[1], rgba[2], strength)?;

    if ctx.dry_run {
        let desc = op.describe();
        ctx.print_output(
            &format!("Would tint {input} with {color_str} at strength {strength}"),
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
        &format!(
            "Tinted with {} at {}: {} → {}",
            color_str, strength, input, output
        ),
        &TintResult {
            input: input.to_string(),
            output,
            color: color_str.to_string(),
            strength,
            width: out.new_width,
            height: out.new_height,
        },
    );
    Ok(0)
}
