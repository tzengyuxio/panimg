use super::common::{require_input, require_output};
use super::CommandResult;
use crate::app::{OverlayArgs, RunContext};
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::ops::overlay::{create_tiled_overlay, OverlayOp};
use panimg_core::ops::position::{resolve_position, Position};
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct OverlayResult {
    input: String,
    layer: String,
    output: String,
    x: i64,
    y: i64,
    opacity: f32,
    output_size: u64,
}

pub fn schema() -> pan_common::schema::CommandSchema {
    OverlayOp::schema()
}

pub fn run(args: &OverlayArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(
        &args.input,
        "panimg overlay <base> --layer <overlay> -o <output>",
    )?;
    let output_path_str = require_output(
        &args.output,
        &args.output_pos,
        "panimg overlay <base> --layer <overlay> -o <output>",
    )?;

    let layer_path_str = args
        .layer
        .as_deref()
        .ok_or_else(|| PanimgError::InvalidArgument {
            message: "missing required argument: --layer".into(),
            suggestion: "usage: panimg overlay <base> --layer <overlay> -o <output>".into(),
        })?;

    let opacity = args.opacity.unwrap_or(1.0);
    let margin = args.margin.unwrap_or(10);

    let input_path = Path::new(input);
    let layer_path = Path::new(layer_path_str);
    let output_path = Path::new(&output_path_str);

    // Decode layer image first (needed for position calculation and tiling)
    let layer_img = CodecRegistry::decode_with_options(layer_path, &ctx.decode_options())?;

    // We need base dimensions for position calculation; decode base for that
    let base_img = CodecRegistry::decode_with_options(input_path, &ctx.decode_options())?;

    let base_w = base_img.width();
    let base_h = base_img.height();
    let layer_w = layer_img.width();
    let layer_h = layer_img.height();

    // Resolve position
    let (x, y) = if let Some(pos_str) = &args.position {
        let pos: Position = pos_str.parse()?;
        resolve_position(pos, base_w, base_h, layer_w, layer_h, margin)
    } else {
        (args.x.unwrap_or(0), args.y.unwrap_or(0))
    };

    // Handle tiling
    let final_layer = if args.tile {
        let spacing = args.spacing.unwrap_or(0);
        create_tiled_overlay(&layer_img, base_w, base_h, opacity, spacing)?
    } else {
        layer_img
    };

    let (final_x, final_y) = if args.tile { (0, 0) } else { (x, y) };

    if ctx.dry_run {
        let plan = serde_json::json!({
            "operation": "overlay",
            "x": final_x,
            "y": final_y,
            "opacity": opacity,
            "tile": args.tile,
        });
        ctx.print_output(
            &format!(
                "Would overlay {} on {} → {} (x={}, y={}, opacity={})",
                layer_path_str, input, output_path_str, final_x, final_y, opacity
            ),
            &plan,
        );
        return Ok(0);
    }

    let overlay_op = OverlayOp::new(final_layer, final_x, final_y, opacity)?;
    let pipeline = Pipeline::new().push(overlay_op);
    let result_img = pipeline.execute(base_img)?;

    let out_format = ImageFormat::from_path_extension(output_path)
        .or_else(|| ImageFormat::from_path(input_path))
        .unwrap_or(ImageFormat::Png);

    let options = EncodeOptions {
        format: out_format,
        quality: args.quality,
        strip_metadata: args.strip,
        resolution: None,
    };

    CodecRegistry::encode(&result_img, output_path, &options)?;

    let output_size = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);

    ctx.print_output(
        &format!(
            "Overlay {} + {} → {} (x={}, y={}, opacity={})",
            input, layer_path_str, output_path_str, final_x, final_y, opacity
        ),
        &OverlayResult {
            input: input.to_string(),
            layer: layer_path_str.to_string(),
            output: output_path_str,
            x: final_x,
            y: final_y,
            opacity,
            output_size,
        },
    );

    Ok(0)
}
