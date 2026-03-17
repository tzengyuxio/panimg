use super::common::{require_input, require_output};
use super::CommandResult;
use crate::app::{RunContext, SmartCropArgs};
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

pub fn schema() -> pan_common::schema::CommandSchema {
    SmartCropOp::schema()
}

pub fn run(args: &SmartCropArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(
        &args.input,
        "panimg smart-crop <input> -o <output> --width W --height H",
    )?;
    let output_path_str = require_output(
        &args.output,
        &args.output_pos,
        "panimg smart-crop <input> -o <output> --width W --height H",
    )?;

    let width = args.width.ok_or_else(|| PanimgError::InvalidArgument {
        message: "missing required argument: --width".into(),
        suggestion: "usage: panimg smart-crop <input> -o <output> --width W --height H".into(),
    })?;
    let height = args.height.ok_or_else(|| PanimgError::InvalidArgument {
        message: "missing required argument: --height".into(),
        suggestion: "usage: panimg smart-crop <input> -o <output> --width W --height H".into(),
    })?;

    let strategy = SmartCropStrategy::parse(args.strategy.as_deref().unwrap_or("entropy"))?;
    let step = args.step;
    let op = SmartCropOp::new(width, height, strategy, step)?;

    let input_path = Path::new(input);
    let output_path = Path::new(&output_path_str);

    if ctx.dry_run {
        let pipeline = Pipeline::new().push(op);
        let plan = pipeline.describe();
        ctx.print_output(
            &format!(
                "Would smart-crop {} → {} ({}x{}, strategy={})",
                input, output_path_str, width, height, strategy
            ),
            &plan,
        );
        return Ok(0);
    }

    let img = CodecRegistry::decode_with_options(input_path, &ctx.decode_options())?;

    // Find best crop position once, then crop directly (avoid double search).
    let (crop_x, crop_y) = op.find_best_crop(&img)?;

    let crop_op = CropOp::new(crop_x, crop_y, width, height)?;
    let pipeline = Pipeline::new().push(crop_op);
    let result_img = pipeline.execute(img)?;

    let out_format = ImageFormat::from_path_extension(output_path)
        .or_else(|| ImageFormat::from_path(input_path))
        .unwrap_or(ImageFormat::Png);

    let options = EncodeOptions {
        format: out_format,
        quality: args.quality,
        strip_metadata: args.strip,
        resolution: None,
    };

    CodecRegistry::encode(&result_img, output_path, &options)?;

    let output_size = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);

    ctx.print_output(
        &format!(
            "Smart-crop {} → {} ({}x{} at ({},{}), strategy={})",
            input, output_path_str, width, height, crop_x, crop_y, strategy
        ),
        &SmartCropResult {
            input: input.to_string(),
            output: output_path_str,
            crop_x,
            crop_y,
            crop_width: width,
            crop_height: height,
            strategy: strategy.to_string(),
            output_size,
        },
    );

    Ok(0)
}
