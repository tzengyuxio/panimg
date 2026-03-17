use super::common::{require_input, require_output, run_pipeline, PipelineInput};
use super::CommandResult;
use crate::app::{BlurArgs, RunContext};
use panimg_core::error::PanimgError;
use panimg_core::ops::blur::{BilateralBlurOp, BlurOp, BoxBlurOp, MedianBlurOp, MotionBlurOp};
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct BlurResult {
    input: String,
    output: String,
    method: String,
    params: serde_json::Value,
    output_size: u64,
}

pub fn schema() -> pan_common::schema::CommandSchema {
    BlurOp::schema()
}

pub fn run(args: &BlurArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(&args.input, "panimg blur <input> -o <output> --sigma 2.0")?;
    let output = require_output(
        &args.output,
        &args.output_pos,
        "panimg blur <input> -o <output> --sigma 2.0",
    )?;

    let method = args.method.as_str();
    let pipeline = build_blur_pipeline(args, method)?;

    let pi = PipelineInput {
        input_path: Path::new(input),
        output_path: Path::new(&output),
        quality: args.quality,
        strip_metadata: args.strip,
    };

    let Some(out) = run_pipeline(&pipeline, &pi, ctx)? else {
        return Ok(0);
    };

    let desc = pipeline.describe();
    let params = desc
        .steps
        .first()
        .map(|s| s.params.clone())
        .unwrap_or_default();

    ctx.print_output(
        &desc
            .steps
            .first()
            .map(|s| s.description.clone())
            .unwrap_or_else(|| format!("{method} blur applied")),
        &BlurResult {
            input: input.to_string(),
            output,
            method: method.to_string(),
            params,
            output_size: out.output_size,
        },
    );
    Ok(0)
}

fn build_blur_pipeline(
    args: &BlurArgs,
    method: &str,
) -> std::result::Result<Pipeline, PanimgError> {
    match method {
        "gaussian" => {
            let sigma = args.sigma.ok_or_else(|| PanimgError::InvalidArgument {
                message: "gaussian blur requires --sigma".into(),
                suggestion: "usage: panimg blur <input> -o <output> --sigma 2.0".into(),
            })?;
            Ok(Pipeline::new().push(BlurOp::new(sigma)?))
        }
        "box" => {
            let radius = args.radius.ok_or_else(|| PanimgError::InvalidArgument {
                message: "box blur requires --radius".into(),
                suggestion: "usage: panimg blur <input> -o <output> --method box --radius 3".into(),
            })?;
            Ok(Pipeline::new().push(BoxBlurOp::new(radius)?))
        }
        "motion" => {
            let angle = args.angle.unwrap_or(0.0);
            let distance = args.distance.ok_or_else(|| PanimgError::InvalidArgument {
                message: "motion blur requires --distance".into(),
                suggestion:
                    "usage: panimg blur <input> -o <output> --method motion --distance 10 --angle 45"
                        .into(),
            })?;
            Ok(Pipeline::new().push(MotionBlurOp::new(angle, distance)?))
        }
        "median" => {
            let radius = args.radius.ok_or_else(|| PanimgError::InvalidArgument {
                message: "median blur requires --radius".into(),
                suggestion: "usage: panimg blur <input> -o <output> --method median --radius 2"
                    .into(),
            })?;
            Ok(Pipeline::new().push(MedianBlurOp::new(radius)?))
        }
        "bilateral" => {
            let sigma = args.sigma.ok_or_else(|| PanimgError::InvalidArgument {
                message: "bilateral blur requires --sigma (spatial sigma)".into(),
                suggestion: "usage: panimg blur <input> -o <output> --method bilateral --sigma 5.0 --sigma-color 50 --radius 5".into(),
            })?;
            let sigma_color = args.sigma_color.ok_or_else(|| PanimgError::InvalidArgument {
                message: "bilateral blur requires --sigma-color".into(),
                suggestion: "usage: panimg blur <input> -o <output> --method bilateral --sigma 5.0 --sigma-color 50 --radius 5".into(),
            })?;
            let radius = args.radius.ok_or_else(|| PanimgError::InvalidArgument {
                message: "bilateral blur requires --radius".into(),
                suggestion: "usage: panimg blur <input> -o <output> --method bilateral --sigma 5.0 --sigma-color 50 --radius 5".into(),
            })?;
            Ok(Pipeline::new().push(BilateralBlurOp::new(radius, sigma, sigma_color)?))
        }
        _ => Err(PanimgError::InvalidArgument {
            message: format!("unknown blur method: '{method}'"),
            suggestion: "supported methods: gaussian, box, motion, median, bilateral".into(),
        }),
    }
}
