use crate::app::{OutputFormat, ResizeArgs};
use crate::output;
use panimg_core::codec::{CodecRegistry, DecodeOptions, EncodeOptions};
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::ops::resize::{FitMode, ResizeFilter, ResizeOp};
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct ResizePlan {
    input: String,
    output: String,
    steps: Vec<panimg_core::ops::OperationDescription>,
    output_format: String,
    quality: Option<u8>,
}

#[derive(Serialize)]
struct ResizeResult {
    input: String,
    output: String,
    original_width: u32,
    original_height: u32,
    new_width: u32,
    new_height: u32,
    output_size: u64,
}

pub fn run(
    args: &ResizeArgs,
    format: OutputFormat,
    dry_run: bool,
    show_schema: bool,
    dpi: Option<f32>,
) -> i32 {
    if show_schema {
        let s = ResizeOp::schema();
        output::print_json(&serde_json::to_value(&s).unwrap());
        return 0;
    }

    let input = match &args.input {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: input".into(),
                suggestion: "usage: panimg resize <input> -o <output> --width <px>".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let output_path_str = match args.output.as_ref().or(args.output_pos.as_ref()) {
        Some(o) => o.clone(),
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: output (-o)".into(),
                suggestion: "usage: panimg resize <input> -o <output> --width <px>".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let input_path = Path::new(input);
    let output_path = Path::new(&output_path_str);

    // Parse fit and filter
    let fit = match FitMode::parse(&args.fit) {
        Ok(f) => f,
        Err(e) => return output::print_error(format, &e),
    };
    let filter = match ResizeFilter::parse(&args.filter) {
        Ok(f) => f,
        Err(e) => return output::print_error(format, &e),
    };

    // Build resize operation
    let resize_op = match ResizeOp::new(args.width, args.height, fit, filter) {
        Ok(op) => op,
        Err(e) => return output::print_error(format, &e),
    };

    let pipeline = Pipeline::new().push(resize_op);

    // Determine output format
    let out_format = ImageFormat::from_path_extension(output_path)
        .or_else(|| ImageFormat::from_path(input_path))
        .unwrap_or(ImageFormat::Png);

    // Dry run
    if dry_run {
        let plan = ResizePlan {
            input: input.clone(),
            output: output_path_str,
            steps: pipeline.describe().steps,
            output_format: out_format.to_string(),
            quality: args.quality,
        };
        output::print_output(
            format,
            &format!("Would resize {} → {}", input, plan.output),
            &plan,
        );
        return 0;
    }

    // Decode input
    let decode_opts = DecodeOptions::with_dpi(dpi);
    let img = match CodecRegistry::decode_with_options(input_path, &decode_opts) {
        Ok(i) => i,
        Err(e) => return output::print_error(format, &e),
    };

    let original_width = img.width();
    let original_height = img.height();

    // Execute pipeline
    let result_img = match pipeline.execute(img) {
        Ok(i) => i,
        Err(e) => return output::print_error(format, &e),
    };

    let new_width = result_img.width();
    let new_height = result_img.height();

    // Encode output
    let options = EncodeOptions {
        format: out_format,
        quality: args.quality,
        strip_metadata: args.strip,
    };

    if let Err(e) = CodecRegistry::encode(&result_img, output_path, &options) {
        return output::print_error(format, &e);
    }

    let output_size = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);

    let result = ResizeResult {
        input: input.clone(),
        output: output_path_str,
        original_width,
        original_height,
        new_width,
        new_height,
        output_size,
    };

    output::print_output(
        format,
        &format!(
            "Resized {} ({}x{}) → {} ({}x{})",
            result.input, original_width, original_height, result.output, new_width, new_height
        ),
        &result,
    );

    0
}
