use super::common::{require_input, require_output, run_pipeline, PipelineInput};
use super::CommandResult;
use crate::app::{DrawArgs, RunContext};
use panimg_core::color::parse_color;
use panimg_core::error::PanimgError;
use panimg_core::ops::draw::{DrawCircleOp, DrawLineOp, DrawRectOp};
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct DrawResult {
    input: String,
    output: String,
    shape: String,
    output_size: u64,
}

pub fn schema() -> pan_common::schema::CommandSchema {
    DrawRectOp::schema()
}

pub fn run(args: &DrawArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(
        &args.input,
        "panimg draw <input> -o <output> --shape rect --x 10 --y 10 --width 100 --height 50",
    )?;
    let output = require_output(
        &args.output,
        &args.output_pos,
        "panimg draw <input> -o <output> --shape rect ...",
    )?;

    let shape = args
        .shape
        .as_deref()
        .ok_or_else(|| PanimgError::InvalidArgument {
            message: "missing required argument: --shape".into(),
            suggestion: "use --shape rect, --shape circle, or --shape line".into(),
        })?;

    let color_str = args.color.as_deref().unwrap_or("red");
    let color = parse_color(color_str)?;
    let fill = args.fill;
    let thickness = args.thickness.unwrap_or(2);

    let pipeline = match shape {
        "rect" => {
            let x = args.x.unwrap_or(0);
            let y = args.y.unwrap_or(0);
            let width = args.width.ok_or_else(|| PanimgError::InvalidArgument {
                message: "rect requires --width".into(),
                suggestion: "e.g. --shape rect --x 10 --y 10 --width 100 --height 50".into(),
            })?;
            let height = args.height.ok_or_else(|| PanimgError::InvalidArgument {
                message: "rect requires --height".into(),
                suggestion: "e.g. --shape rect --x 10 --y 10 --width 100 --height 50".into(),
            })?;
            let op = DrawRectOp::new(x, y, width, height, color, fill, thickness)?;
            Pipeline::new().push(op)
        }
        "circle" => {
            let cx = args.cx.or(args.x).unwrap_or(0);
            let cy = args.cy.or(args.y).unwrap_or(0);
            let radius = args.radius.ok_or_else(|| PanimgError::InvalidArgument {
                message: "circle requires --radius".into(),
                suggestion: "e.g. --shape circle --cx 50 --cy 50 --radius 30".into(),
            })?;
            let op = DrawCircleOp::new(cx, cy, radius, color, fill, thickness)?;
            Pipeline::new().push(op)
        }
        "line" => {
            let x1 = args.x1.or(args.x).unwrap_or(0);
            let y1 = args.y1.or(args.y).unwrap_or(0);
            let x2 = args.x2.ok_or_else(|| PanimgError::InvalidArgument {
                message: "line requires --x2".into(),
                suggestion: "e.g. --shape line --x1 0 --y1 0 --x2 100 --y2 100".into(),
            })?;
            let y2 = args.y2.ok_or_else(|| PanimgError::InvalidArgument {
                message: "line requires --y2".into(),
                suggestion: "e.g. --shape line --x1 0 --y1 0 --x2 100 --y2 100".into(),
            })?;
            Pipeline::new().push(DrawLineOp::new(x1, y1, x2, y2, color, thickness))
        }
        _ => {
            return Err(PanimgError::InvalidArgument {
                message: format!("unknown shape: '{shape}'"),
                suggestion: "use --shape rect, --shape circle, or --shape line".into(),
            });
        }
    };

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
        &format!("Drew {} on {} → {}", shape, input, output),
        &DrawResult {
            input: input.to_string(),
            output,
            shape: shape.to_string(),
            output_size: out.output_size,
        },
    );
    Ok(0)
}
