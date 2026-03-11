use crate::app::{OutputFormat, SepiaArgs};
use crate::output;
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::ops::color::SepiaOp;
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct SepiaResult {
    input: String,
    output: String,
    intensity: f32,
    width: u32,
    height: u32,
}

pub fn run(args: &SepiaArgs, format: OutputFormat, dry_run: bool, show_schema: bool) -> i32 {
    if show_schema {
        let s = SepiaOp::schema();
        output::print_json(&serde_json::to_value(&s).unwrap());
        return 0;
    }

    let input = match &args.input {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: input".into(),
                suggestion: "usage: panimg sepia <input> -o <output>".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let output_path_str = match args.output.as_ref().or(args.output_pos.as_ref()) {
        Some(o) => o.clone(),
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: output (-o)".into(),
                suggestion: "usage: panimg sepia <input> -o <output>".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let intensity = args.intensity.unwrap_or(1.0);

    let op = match SepiaOp::new(intensity) {
        Ok(o) => o,
        Err(e) => return output::print_error(format, &e),
    };

    if dry_run {
        let desc = op.describe();
        output::print_output(
            format,
            &format!("Would apply sepia tone to {input} at intensity {intensity}"),
            &desc,
        );
        return 0;
    }

    let input_path = Path::new(input);
    let output_path = Path::new(&output_path_str);

    let img = match CodecRegistry::decode(input_path) {
        Ok(i) => i,
        Err(e) => return output::print_error(format, &e),
    };

    let pipeline = Pipeline::new().push(op);
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

    let result = SepiaResult {
        input: input.clone(),
        output: output_path_str,
        intensity,
        width: result_img.width(),
        height: result_img.height(),
    };

    output::print_output(
        format,
        &format!("Sepia applied: {} → {}", result.input, result.output),
        &result,
    );

    0
}
