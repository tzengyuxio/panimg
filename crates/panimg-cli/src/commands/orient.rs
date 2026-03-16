use crate::app::{AutoOrientArgs, RunContext};
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::ops::orient::AutoOrientOp;
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct AutoOrientResult {
    input: String,
    output: String,
    new_width: u32,
    new_height: u32,
    output_size: u64,
}

pub fn run(args: &AutoOrientArgs, ctx: &RunContext) -> i32 {
    if ctx.schema {
        let s = AutoOrientOp::schema();
        ctx.print_json(&serde_json::to_value(&s).unwrap());
        return 0;
    }

    let input = match &args.input {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: input".into(),
                suggestion: "usage: panimg auto-orient <input> -o <output>".into(),
            };
            return ctx.print_error(&err);
        }
    };

    let output_path_str = match args.output.as_ref().or(args.output_pos.as_ref()) {
        Some(o) => o.clone(),
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: output (-o)".into(),
                suggestion: "usage: panimg auto-orient <input> -o <output>".into(),
            };
            return ctx.print_error(&err);
        }
    };

    let input_path = Path::new(input);
    let output_path = Path::new(&output_path_str);

    let orient_op = AutoOrientOp::from_path(input_path);
    let pipeline = Pipeline::new().push(orient_op);

    if ctx.dry_run {
        let plan = pipeline.describe();
        ctx.print_output(
            &format!("Would auto-orient {} → {}", input, output_path_str),
            &plan,
        );
        return 0;
    }

    let img = match CodecRegistry::decode_with_options(input_path, &ctx.decode_options()) {
        Ok(i) => i,
        Err(e) => return ctx.print_error(&e),
    };

    let result_img = match pipeline.execute(img) {
        Ok(i) => i,
        Err(e) => return ctx.print_error(&e),
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
        return ctx.print_error(&e);
    }

    let output_size = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);

    let result = AutoOrientResult {
        input: input.clone(),
        output: output_path_str,
        new_width: result_img.width(),
        new_height: result_img.height(),
        output_size,
    };

    ctx.print_output(
        &format!(
            "Auto-oriented {} → {} ({}x{})",
            result.input, result.output, result.new_width, result.new_height
        ),
        &result,
    );

    0
}
