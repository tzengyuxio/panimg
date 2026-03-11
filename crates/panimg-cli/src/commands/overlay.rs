use crate::app::{OutputFormat, OverlayArgs};
use crate::output;
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::ops::overlay::{create_tiled_overlay, resolve_position, OverlayOp};
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

pub fn run(args: &OverlayArgs, format: OutputFormat, dry_run: bool, show_schema: bool) -> i32 {
    if show_schema {
        let s = OverlayOp::schema();
        output::print_json(&serde_json::to_value(&s).unwrap());
        return 0;
    }

    let input = match &args.input {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: input".into(),
                suggestion: "usage: panimg overlay <base> --layer <overlay> -o <output>".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let layer_path_str = match &args.layer {
        Some(l) => l,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: --layer".into(),
                suggestion: "usage: panimg overlay <base> --layer <overlay> -o <output>".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let output_path_str = match args.output.as_ref().or(args.output_pos.as_ref()) {
        Some(o) => o.clone(),
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: output (-o)".into(),
                suggestion: "usage: panimg overlay <base> --layer <overlay> -o <output>".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let opacity = args.opacity.unwrap_or(1.0);
    let margin = args.margin.unwrap_or(10);

    let input_path = Path::new(input);
    let layer_path = Path::new(layer_path_str);
    let output_path = Path::new(&output_path_str);

    // Decode layer image first (needed for position calculation and tiling)
    let layer_img = match CodecRegistry::decode(layer_path) {
        Ok(i) => i,
        Err(e) => return output::print_error(format, &e),
    };

    // We need base dimensions for position calculation; decode base for that
    let base_img = match CodecRegistry::decode(input_path) {
        Ok(i) => i,
        Err(e) => return output::print_error(format, &e),
    };

    let base_w = base_img.width();
    let base_h = base_img.height();
    let layer_w = layer_img.width();
    let layer_h = layer_img.height();

    // Resolve position
    let (x, y) = if let Some(pos) = &args.position {
        match resolve_position(pos, base_w, base_h, layer_w, layer_h, margin) {
            Ok(p) => p,
            Err(e) => return output::print_error(format, &e),
        }
    } else {
        (args.x.unwrap_or(0), args.y.unwrap_or(0))
    };

    // Handle tiling
    let final_layer = if args.tile {
        let spacing = args.spacing.unwrap_or(0);
        match create_tiled_overlay(&layer_img, base_w, base_h, opacity, spacing) {
            Ok(tiled) => tiled,
            Err(e) => return output::print_error(format, &e),
        }
    } else {
        layer_img
    };

    let (final_x, final_y) = if args.tile { (0, 0) } else { (x, y) };

    if dry_run {
        let plan = serde_json::json!({
            "operation": "overlay",
            "x": final_x,
            "y": final_y,
            "opacity": opacity,
            "tile": args.tile,
        });
        output::print_output(
            format,
            &format!(
                "Would overlay {} on {} → {} (x={}, y={}, opacity={})",
                layer_path_str, input, output_path_str, final_x, final_y, opacity
            ),
            &plan,
        );
        return 0;
    }

    let overlay_op = match OverlayOp::new(final_layer, final_x, final_y, opacity) {
        Ok(op) => op,
        Err(e) => return output::print_error(format, &e),
    };

    let pipeline = Pipeline::new().push(overlay_op);

    let result_img = match pipeline.execute(base_img) {
        Ok(i) => i,
        Err(e) => return output::print_error(format, &e),
    };

    let out_format = ImageFormat::from_path_extension(output_path)
        .or_else(|| ImageFormat::from_path(input_path))
        .unwrap_or(ImageFormat::Png);

    let options = EncodeOptions {
        format: out_format,
        quality: args.quality,
        strip_metadata: args.strip,
    };

    if let Err(e) = CodecRegistry::encode(&result_img, output_path, &options) {
        return output::print_error(format, &e);
    }

    let output_size = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);

    let result = OverlayResult {
        input: input.clone(),
        layer: layer_path_str.clone(),
        output: output_path_str,
        x: final_x,
        y: final_y,
        opacity,
        output_size,
    };

    output::print_output(
        format,
        &format!(
            "Overlay {} + {} → {} (x={}, y={}, opacity={})",
            result.input, result.layer, result.output, result.x, result.y, result.opacity
        ),
        &result,
    );

    0
}
