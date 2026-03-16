use crate::app::{GrayscaleArgs, RunContext};
use crate::output;
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::ops::grayscale::GrayscaleOp;
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct GrayscaleResult {
    input: String,
    output: String,
    output_size: u64,
}

pub fn run(args: &GrayscaleArgs, ctx: &RunContext) -> i32 {
    if ctx.schema {
        let s = GrayscaleOp::schema();
        output::print_json(&serde_json::to_value(&s).unwrap());
        return 0;
    }

    let input = match &args.input {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: input".into(),
                suggestion: "usage: panimg grayscale <input> -o <output>".into(),
            };
            return output::print_error(ctx.format, &err);
        }
    };

    let output_path_str = match args.output.as_ref().or(args.output_pos.as_ref()) {
        Some(o) => o.clone(),
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: output (-o)".into(),
                suggestion: "usage: panimg grayscale <input> -o <output>".into(),
            };
            return output::print_error(ctx.format, &err);
        }
    };

    let pipeline = Pipeline::new().push(GrayscaleOp::new());
    let input_path = Path::new(input);
    let output_path = Path::new(&output_path_str);

    if ctx.dry_run {
        let plan = pipeline.describe();
        output::print_output(
            ctx.format,
            &format!("Would convert to grayscale {} → {}", input, output_path_str),
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

    let result = GrayscaleResult {
        input: input.clone(),
        output: output_path_str,
        output_size,
    };

    output::print_output(
        ctx.format,
        &format!("Grayscale {} → {}", result.input, result.output),
        &result,
    );

    0
}
