use crate::app::{RotateArgs, RunContext};
use crate::output;
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::color::parse_color;
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::ops::rotate::{RotateAngle, RotateOp};
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct RotateResult {
    input: String,
    output: String,
    angle: f64,
    new_width: u32,
    new_height: u32,
    output_size: u64,
}

/// Determine the default background color string based on the output format.
/// - Formats that support alpha (PNG, WebP, TIFF, GIF) → transparent
/// - Opaque formats (JPEG, BMP) → white
fn default_background_str(format: &ImageFormat) -> &'static str {
    match format {
        ImageFormat::Jpeg | ImageFormat::Bmp => "white",
        _ => "transparent",
    }
}

pub fn run(args: &RotateArgs, ctx: &RunContext) -> i32 {
    if ctx.schema {
        let s = RotateOp::schema();
        output::print_json(&serde_json::to_value(&s).unwrap());
        return 0;
    }

    let input = match &args.input {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: input".into(),
                suggestion: "usage: panimg rotate <input> -o <output> --angle 90".into(),
            };
            return output::print_error(ctx.format, &err);
        }
    };

    let output_path_str = match args.output.as_ref().or(args.output_pos.as_ref()) {
        Some(o) => o.clone(),
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: output (-o)".into(),
                suggestion: "usage: panimg rotate <input> -o <output> --angle 90".into(),
            };
            return output::print_error(ctx.format, &err);
        }
    };

    let angle_str = match &args.angle {
        Some(a) => a,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: --angle".into(),
                suggestion: "usage: panimg rotate <input> -o <output> --angle 90".into(),
            };
            return output::print_error(ctx.format, &err);
        }
    };

    let angle = match RotateAngle::parse(angle_str) {
        Ok(a) => a,
        Err(e) => return output::print_error(ctx.format, &e),
    };

    let input_path = Path::new(input);
    let output_path = Path::new(&output_path_str);

    // Determine output format for default background color
    let out_format = ImageFormat::from_path_extension(output_path)
        .or_else(|| ImageFormat::from_path(input_path))
        .unwrap_or(ImageFormat::Png);

    // Parse background color
    let bg_str = args
        .background
        .as_deref()
        .unwrap_or_else(|| default_background_str(&out_format));
    let background = match parse_color(bg_str) {
        Ok(c) => c,
        Err(e) => return output::print_error(ctx.format, &e),
    };

    let rotate_op = RotateOp::new(angle).with_background(background);
    let pipeline = Pipeline::new().push(rotate_op);

    if ctx.dry_run {
        let plan = pipeline.describe();
        output::print_output(
            ctx.format,
            &format!("Would rotate {} → {}", input, output_path_str),
            &plan,
        );
        return 0;
    }

    let img = match CodecRegistry::decode_with_options(input_path, &ctx.decode_options()) {
        Ok(i) => i,
        Err(e) => return output::print_error(ctx.format, &e),
    };

    let result_img = match pipeline.execute(img) {
        Ok(i) => i,
        Err(e) => return output::print_error(ctx.format, &e),
    };

    let options = EncodeOptions {
        format: out_format,
        quality: args.quality,
        strip_metadata: args.strip,
        resolution: None,
    };

    if let Err(e) = CodecRegistry::encode(&result_img, output_path, &options) {
        return output::print_error(ctx.format, &e);
    }

    let output_size = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);

    let result = RotateResult {
        input: input.clone(),
        output: output_path_str,
        angle: angle.degrees_f64(),
        new_width: result_img.width(),
        new_height: result_img.height(),
        output_size,
    };

    output::print_output(
        ctx.format,
        &format!(
            "Rotated {} → {} ({}°, {}x{})",
            result.input, result.output, result.angle, result.new_width, result.new_height
        ),
        &result,
    );

    0
}
