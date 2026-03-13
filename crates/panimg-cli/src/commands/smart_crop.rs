use crate::app::{OutputFormat, SmartCropArgs};
use crate::output;
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::ops::crop::CropOp;
use panimg_core::ops::smart_crop::{SmartCropOp, SmartCropStrategy};
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct SmartCropResult {
    input: String,
    output: String,
    crop_x: u32,
    crop_y: u32,
    crop_width: u32,
    crop_height: u32,
    strategy: String,
    output_size: u64,
}

pub fn run(args: &SmartCropArgs, format: OutputFormat, dry_run: bool, show_schema: bool) -> i32 {
    if show_schema {
        let s = SmartCropOp::schema();
        output::print_json(&serde_json::to_value(&s).unwrap());
        return 0;
    }

    let input = match &args.input {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: input".into(),
                suggestion: "usage: panimg smart-crop <input> -o <output> --width W --height H"
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
                suggestion: "usage: panimg smart-crop <input> -o <output> --width W --height H"
                    .into(),
            };
            return output::print_error(format, &err);
        }
    };

    let width = match args.width {
        Some(w) => w,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: --width".into(),
                suggestion: "usage: panimg smart-crop <input> -o <output> --width W --height H"
                    .into(),
            };
            return output::print_error(format, &err);
        }
    };

    let height = match args.height {
        Some(h) => h,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: --height".into(),
                suggestion: "usage: panimg smart-crop <input> -o <output> --width W --height H"
                    .into(),
            };
            return output::print_error(format, &err);
        }
    };

    let strategy = match SmartCropStrategy::parse(args.strategy.as_deref().unwrap_or("entropy")) {
        Ok(s) => s,
        Err(e) => return output::print_error(format, &e),
    };

    let step = args.step;

    let op = match SmartCropOp::new(width, height, strategy, step) {
        Ok(op) => op,
        Err(e) => return output::print_error(format, &e),
    };

    let input_path = Path::new(input);
    let output_path = Path::new(&output_path_str);

    if dry_run {
        let pipeline = Pipeline::new().push(op);
        let plan = pipeline.describe();
        output::print_output(
            format,
            &format!(
                "Would smart-crop {} → {} ({}x{}, strategy={})",
                input, output_path_str, width, height, strategy
            ),
            &plan,
        );
        return 0;
    }

    let img = match CodecRegistry::decode(input_path) {
        Ok(i) => i,
        Err(e) => return output::print_error(format, &e),
    };

    // Find best crop position once, then crop directly (avoid double search).
    let (crop_x, crop_y) = match op.find_best_crop(&img) {
        Ok(pos) => pos,
        Err(e) => return output::print_error(format, &e),
    };

    let crop_op = match CropOp::new(crop_x, crop_y, width, height) {
        Ok(op) => op,
        Err(e) => return output::print_error(format, &e),
    };

    let pipeline = Pipeline::new().push(crop_op);
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

    let output_size = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);

    let result = SmartCropResult {
        input: input.clone(),
        output: output_path_str,
        crop_x,
        crop_y,
        crop_width: width,
        crop_height: height,
        strategy: strategy.to_string(),
        output_size,
    };

    output::print_output(
        format,
        &format!(
            "Smart-crop {} → {} ({}x{} at ({},{}), strategy={})",
            result.input, result.output, width, height, crop_x, crop_y, strategy
        ),
        &result,
    );

    0
}
