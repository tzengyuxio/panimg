use crate::app::{RunContext, SharpenArgs};
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::ops::sharpen::SharpenOp;
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct SharpenResult {
    input: String,
    output: String,
    sigma: f32,
    threshold: i32,
    output_size: u64,
}

pub fn run(args: &SharpenArgs, ctx: &RunContext) -> i32 {
    if ctx.schema {
        let s = SharpenOp::schema();
        ctx.print_json(&serde_json::to_value(&s).unwrap());
        return 0;
    }

    let input = match &args.input {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: input".into(),
                suggestion: "usage: panimg sharpen <input> -o <output> --sigma 1.0".into(),
            };
            return ctx.print_error(&err);
        }
    };

    let output_path_str = match args.output.as_ref().or(args.output_pos.as_ref()) {
        Some(o) => o.clone(),
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: output (-o)".into(),
                suggestion: "usage: panimg sharpen <input> -o <output> --sigma 1.0".into(),
            };
            return ctx.print_error(&err);
        }
    };

    let sigma = match args.sigma {
        Some(s) => s,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: --sigma".into(),
                suggestion: "usage: panimg sharpen <input> -o <output> --sigma 1.0".into(),
            };
            return ctx.print_error(&err);
        }
    };

    let threshold = args.threshold.unwrap_or(0);

    let sharpen_op = match SharpenOp::new(sigma, threshold) {
        Ok(op) => op,
        Err(e) => return ctx.print_error(&e),
    };

    let pipeline = Pipeline::new().push(sharpen_op);
    let input_path = Path::new(input);
    let output_path = Path::new(&output_path_str);

    if ctx.dry_run {
        let plan = pipeline.describe();
        ctx.print_output(
            &format!(
                "Would sharpen {} → {} (sigma={}, threshold={})",
                input, output_path_str, sigma, threshold
            ),
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

    let result = SharpenResult {
        input: input.clone(),
        output: output_path_str,
        sigma,
        threshold,
        output_size,
    };

    ctx.print_output(
        &format!(
            "Sharpened {} → {} (sigma={}, threshold={})",
            result.input, result.output, sigma, threshold
        ),
        &result,
    );

    0
}
