use crate::app::{OutputFormat, TintArgs};
use crate::output;
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::color::parse_color;
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::ops::color::TintOp;
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct TintResult {
    input: String,
    output: String,
    color: String,
    strength: f32,
    width: u32,
    height: u32,
}

pub fn run(args: &TintArgs, format: OutputFormat, dry_run: bool, show_schema: bool) -> i32 {
    if show_schema {
        let s = TintOp::schema();
        output::print_json(&serde_json::to_value(&s).unwrap());
        return 0;
    }

    let input = match &args.input {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: input".into(),
                suggestion: "usage: panimg tint <input> -o <output> --color red --strength 0.5"
                    .into(),
            };
            return output::print_error(format, &err);
        }
    };

    let output_path_str = match args.output.as_ref().or(args.output_pos.as_ref()) {
        Some(o) => o.clone(),
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: output (-o)".into(),
                suggestion: "usage: panimg tint <input> -o <output> --color red".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let color_str = match &args.color {
        Some(c) => c.clone(),
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: --color".into(),
                suggestion: "use --color red, --color '#FF0000', or --color '255,0,0'".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let rgba = match parse_color(&color_str) {
        Ok(c) => c,
        Err(e) => return output::print_error(format, &e),
    };

    let strength = args.strength.unwrap_or(0.5);

    let op = match TintOp::new(rgba[0], rgba[1], rgba[2], strength) {
        Ok(o) => o,
        Err(e) => return output::print_error(format, &e),
    };

    if dry_run {
        let desc = op.describe();
        output::print_output(
            format,
            &format!("Would tint {input} with {color_str} at strength {strength}"),
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
        resolution: None,
    };

    if let Err(e) = CodecRegistry::encode(&result_img, output_path, &options) {
        return output::print_error(format, &e);
    }

    let result = TintResult {
        input: input.clone(),
        output: output_path_str,
        color: color_str,
        strength,
        width: result_img.width(),
        height: result_img.height(),
    };

    output::print_output(
        format,
        &format!(
            "Tinted with {} at {}: {} → {}",
            result.color, result.strength, result.input, result.output
        ),
        &result,
    );

    0
}
