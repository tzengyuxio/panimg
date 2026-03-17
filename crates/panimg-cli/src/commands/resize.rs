use super::common::{require_input, require_output, run_pipeline, PipelineInput};
use super::CommandResult;
use crate::app::{ResizeArgs, RunContext};
use panimg_core::format::ImageFormat;
use panimg_core::ops::resize::{FitMode, ResizeFilter, ResizeOp};
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct ResizePlan {
    input: String,
    output: String,
    steps: Vec<panimg_core::ops::OperationDescription>,
    output_format: String,
    quality: Option<u8>,
}

#[derive(Serialize)]
struct ResizeResult {
    input: String,
    output: String,
    original_width: u32,
    original_height: u32,
    new_width: u32,
    new_height: u32,
    output_size: u64,
}

pub fn schema() -> pan_common::schema::CommandSchema {
    ResizeOp::schema()
}

pub fn run(args: &ResizeArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(
        &args.input,
        "panimg resize <input> -o <output> --width <px>",
    )?;
    let output = require_output(
        &args.output,
        &args.output_pos,
        "panimg resize <input> -o <output> --width <px>",
    )?;

    let input_path = Path::new(input);
    let output_path = Path::new(&output);

    let fit = FitMode::parse(&args.fit)?;
    let filter = ResizeFilter::parse(&args.filter)?;
    let resize_op = ResizeOp::new(args.width, args.height, fit, filter)?;
    let pipeline = Pipeline::new().push(resize_op);

    // Determine output format
    let out_format = ImageFormat::from_path_extension(output_path)
        .or_else(|| ImageFormat::from_path(input_path))
        .unwrap_or(ImageFormat::Png);

    // Custom dry-run with ResizePlan
    if ctx.dry_run {
        let plan = ResizePlan {
            input: input.to_string(),
            output: output.clone(),
            steps: pipeline.describe().steps,
            output_format: out_format.to_string(),
            quality: args.quality,
        };
        ctx.print_output(&format!("Would resize {} → {}", input, plan.output), &plan);
        return Ok(0);
    }

    let pi = PipelineInput {
        input_path,
        output_path,
        quality: args.quality,
        strip_metadata: args.strip,
    };

    let Some(out) = run_pipeline(&pipeline, &pi, ctx)? else {
        return Ok(0);
    };

    ctx.print_output(
        &format!(
            "Resized {} ({}x{}) → {} ({}x{})",
            input, out.original_width, out.original_height, output, out.new_width, out.new_height
        ),
        &ResizeResult {
            input: input.to_string(),
            output,
            original_width: out.original_width,
            original_height: out.original_height,
            new_width: out.new_width,
            new_height: out.new_height,
            output_size: out.output_size,
        },
    );
    Ok(0)
}
