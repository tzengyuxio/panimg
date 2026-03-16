use crate::app::{FlipArgs, RunContext};
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::ops::flip::{FlipDirection, FlipOp};
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct FlipResult {
    input: String,
    output: String,
    direction: String,
    output_size: u64,
}

pub fn run(args: &FlipArgs, ctx: &RunContext) -> i32 {
    if ctx.schema {
        let s = FlipOp::schema();
        ctx.print_json(&serde_json::to_value(&s).unwrap());
        return 0;
    }

    let input = match &args.input {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: input".into(),
                suggestion: "usage: panimg flip <input> -o <output> --direction horizontal".into(),
            };
            return ctx.print_error(&err);
        }
    };

    let output_path_str = match args.output.as_ref().or(args.output_pos.as_ref()) {
        Some(o) => o.clone(),
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: output (-o)".into(),
                suggestion: "usage: panimg flip <input> -o <output> --direction horizontal".into(),
            };
            return ctx.print_error(&err);
        }
    };

    let direction_str = match &args.direction {
        Some(d) => d,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: --direction".into(),
                suggestion: "usage: panimg flip <input> -o <output> --direction horizontal".into(),
            };
            return ctx.print_error(&err);
        }
    };

    let direction = match FlipDirection::parse(direction_str) {
        Ok(d) => d,
        Err(e) => return ctx.print_error(&e),
    };

    let flip_op = FlipOp::new(direction);
    let pipeline = Pipeline::new().push(flip_op);

    let input_path = Path::new(input);
    let output_path = Path::new(&output_path_str);

    if ctx.dry_run {
        let plan = pipeline.describe();
        ctx.print_output(
            &format!("Would flip {} → {}", input, output_path_str),
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

    let result = FlipResult {
        input: input.clone(),
        output: output_path_str,
        direction: match direction {
            FlipDirection::Horizontal => "horizontal",
            FlipDirection::Vertical => "vertical",
        }
        .into(),
        output_size,
    };

    ctx.print_output(
        &format!(
            "Flipped {} → {} ({})",
            result.input, result.output, result.direction
        ),
        &result,
    );

    0
}
