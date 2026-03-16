use crate::app::{CropArgs, RunContext};
use crate::output;
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::ops::crop::CropOp;
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct CropResult {
    input: String,
    output: String,
    x: u32,
    y: u32,
    crop_width: u32,
    crop_height: u32,
    output_size: u64,
}

pub fn run(args: &CropArgs, ctx: &RunContext) -> i32 {
    if ctx.schema {
        let s = CropOp::schema();
        output::print_json(&serde_json::to_value(&s).unwrap());
        return 0;
    }

    let input = match &args.input {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: input".into(),
                suggestion:
                    "usage: panimg crop <input> -o <output> --x 0 --y 0 --width 100 --height 100"
                        .into(),
            };
            return output::print_error(ctx.format, &err);
        }
    };

    let output_path_str = match args.output.as_ref().or(args.output_pos.as_ref()) {
        Some(o) => o.clone(),
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: output (-o)".into(),
                suggestion: "usage: panimg crop <input> -o <output> --width 100 --height 100"
                    .into(),
            };
            return output::print_error(ctx.format, &err);
        }
    };

    let width = match args.width {
        Some(w) => w,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: --width".into(),
                suggestion: "specify crop dimensions with --width and --height".into(),
            };
            return output::print_error(ctx.format, &err);
        }
    };

    let height = match args.height {
        Some(h) => h,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: --height".into(),
                suggestion: "specify crop dimensions with --width and --height".into(),
            };
            return output::print_error(ctx.format, &err);
        }
    };

    let crop_op = match CropOp::new(args.x, args.y, width, height) {
        Ok(op) => op,
        Err(e) => return output::print_error(ctx.format, &e),
    };

    let pipeline = Pipeline::new().push(crop_op);

    let input_path = Path::new(input);
    let output_path = Path::new(&output_path_str);

    if ctx.dry_run {
        let plan = pipeline.describe();
        output::print_output(
            ctx.format,
            &format!("Would crop {} → {}", input, output_path_str),
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

    let out_format = ImageFormat::from_path_extension(output_path)
        .or_else(|| ImageFormat::from_path(input_path))
        .unwrap_or(ImageFormat::Png);

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

    let result = CropResult {
        input: input.clone(),
        output: output_path_str,
        x: args.x,
        y: args.y,
        crop_width: width,
        crop_height: height,
        output_size,
    };

    output::print_output(
        ctx.format,
        &format!(
            "Cropped {} → {} ({}x{} at {}, {})",
            result.input, result.output, width, height, args.x, args.y
        ),
        &result,
    );

    0
}
