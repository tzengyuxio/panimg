use crate::app::{BlurArgs, OutputFormat};
use crate::output;
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::ops::blur::BlurOp;
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct BlurResult {
    input: String,
    output: String,
    sigma: f32,
    output_size: u64,
}

pub fn run(args: &BlurArgs, format: OutputFormat, dry_run: bool, show_schema: bool) -> i32 {
    if show_schema {
        let s = BlurOp::schema();
        output::print_json(&serde_json::to_value(&s).unwrap());
        return 0;
    }

    let input = match &args.input {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: input".into(),
                suggestion: "usage: panimg blur <input> -o <output> --sigma 2.0".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let output_path_str = match args.output.as_ref().or(args.output_pos.as_ref()) {
        Some(o) => o.clone(),
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: output (-o)".into(),
                suggestion: "usage: panimg blur <input> -o <output> --sigma 2.0".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let sigma = match args.sigma {
        Some(s) => s,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: --sigma".into(),
                suggestion: "usage: panimg blur <input> -o <output> --sigma 2.0".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let blur_op = match BlurOp::new(sigma) {
        Ok(op) => op,
        Err(e) => return output::print_error(format, &e),
    };

    let pipeline = Pipeline::new().push(blur_op);
    let input_path = Path::new(input);
    let output_path = Path::new(&output_path_str);

    if dry_run {
        let plan = pipeline.describe();
        output::print_output(
            format,
            &format!(
                "Would blur {} → {} (sigma={})",
                input, output_path_str, sigma
            ),
            &plan,
        );
        return 0;
    }

    let img = match CodecRegistry::decode(input_path) {
        Ok(i) => i,
        Err(e) => return output::print_error(format, &e),
    };

    let result_img = match pipeline.execute(img) {
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

    let result = BlurResult {
        input: input.clone(),
        output: output_path_str,
        sigma,
        output_size,
    };

    output::print_output(
        format,
        &format!(
            "Blurred {} → {} (sigma={})",
            result.input, result.output, sigma
        ),
        &result,
    );

    0
}
