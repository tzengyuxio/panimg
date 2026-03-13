use crate::app::{OutputFormat, SaturateArgs};
use crate::output;
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::ops::color::SaturateOp;
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct SaturateResult {
    input: String,
    output: String,
    factor: f32,
    width: u32,
    height: u32,
}

pub fn run(args: &SaturateArgs, format: OutputFormat, dry_run: bool, show_schema: bool) -> i32 {
    if show_schema {
        let s = SaturateOp::schema();
        output::print_json(&serde_json::to_value(&s).unwrap());
        return 0;
    }

    let input = match &args.input {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: input".into(),
                suggestion: "usage: panimg saturate <input> -o <output> --factor 1.5".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let output_path_str = match args.output.as_ref().or(args.output_pos.as_ref()) {
        Some(o) => o.clone(),
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: output (-o)".into(),
                suggestion: "usage: panimg saturate <input> -o <output> --factor 1.5".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let factor = match args.factor {
        Some(f) => f,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: --factor".into(),
                suggestion: "use --factor 1.5 to increase saturation or --factor 0.5 to decrease"
                    .into(),
            };
            return output::print_error(format, &err);
        }
    };

    let op = match SaturateOp::new(factor) {
        Ok(o) => o,
        Err(e) => return output::print_error(format, &e),
    };

    if dry_run {
        let desc = op.describe();
        output::print_output(
            format,
            &format!("Would adjust saturation of {input} by factor {factor}"),
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

    let result = SaturateResult {
        input: input.clone(),
        output: output_path_str,
        factor,
        width: result_img.width(),
        height: result_img.height(),
    };

    output::print_output(
        format,
        &format!(
            "Saturation adjusted {}x: {} → {}",
            factor, result.input, result.output
        ),
        &result,
    );

    0
}
