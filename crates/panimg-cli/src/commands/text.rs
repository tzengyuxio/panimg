use super::common::{require_input, require_output, run_pipeline, PipelineInput};
use super::CommandResult;
use crate::app::{RunContext, TextArgs};
use panimg_core::color::parse_color;
use panimg_core::error::PanimgError;
use panimg_core::ops::position::Position;
use panimg_core::ops::text::DrawTextOp;
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct TextResult {
    input: String,
    output: String,
    content: String,
    size: f32,
    output_size: u64,
}

pub fn schema() -> pan_common::schema::CommandSchema {
    DrawTextOp::schema()
}

pub fn run(args: &TextArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(
        &args.input,
        "panimg text <input> -o <output> --content \"Hello\"",
    )?;
    let output = require_output(
        &args.output,
        &args.output_pos,
        "panimg text <input> -o <output> --content \"Hello\"",
    )?;

    let content = args
        .content
        .as_deref()
        .ok_or_else(|| PanimgError::InvalidArgument {
            message: "missing required argument: --content".into(),
            suggestion: "usage: panimg text <input> -o <output> --content \"Hello\"".into(),
        })?;

    let size = args.size.unwrap_or(24.0);
    let color_str = args.color.as_deref().unwrap_or("white");
    let color = parse_color(color_str)?;
    let margin = args.margin.unwrap_or(10);

    let position = args
        .position
        .as_deref()
        .map(|s| s.parse::<Position>())
        .transpose()?;

    let text_op = DrawTextOp::new(
        content.to_string(),
        args.font.as_deref(),
        size,
        color,
        args.x,
        args.y,
        position,
        margin,
    )?;

    // Custom dry-run to show text details
    if ctx.dry_run {
        let desc = text_op.describe();
        ctx.print_output(
            &format!(
                "Would draw text \"{}\" on {} → {} (size={})",
                content, input, output, size
            ),
            &desc,
        );
        return Ok(0);
    }

    let pipeline = Pipeline::new().push(text_op);
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
            "Text \"{}\" → {} (size={}, {})",
            content,
            output,
            size,
            humanize_bytes(out.output_size)
        ),
        &TextResult {
            input: input.to_string(),
            output,
            content: content.to_string(),
            size,
            output_size: out.output_size,
        },
    );
    Ok(0)
}

fn humanize_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
