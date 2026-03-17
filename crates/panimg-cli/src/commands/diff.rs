use super::common::require_input;
use super::CommandResult;
use crate::app::{DiffArgs, RunContext};
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::format::ImageFormat;
use panimg_core::ops::diff;
use std::path::Path;

pub fn run(args: &DiffArgs, ctx: &RunContext) -> CommandResult {
    let input_a = require_input(
        &args.input_a,
        "panimg diff <image_a> <image_b> [-o diff.png]",
    )?;
    let input_b = require_input(
        &args.input_b,
        "panimg diff <image_a> <image_b> [-o diff.png]",
    )?;

    let threshold = args.threshold.unwrap_or(0);
    let path_a = Path::new(input_a);
    let path_b = Path::new(input_b);

    if ctx.dry_run {
        let plan = serde_json::json!({
            "operation": "diff",
            "image_a": input_a,
            "image_b": input_b,
            "threshold": threshold,
        });
        ctx.print_output(&format!("Would compare {} vs {}", input_a, input_b), &plan);
        return Ok(0);
    }

    let img_a = CodecRegistry::decode_with_options(path_a, &ctx.decode_options())?;
    let img_b = CodecRegistry::decode_with_options(path_b, &ctx.decode_options())?;

    diff::validate_inputs(&img_a, &img_b)?;

    let (result, diff_img) = diff::compare(&img_a, &img_b, threshold)?;

    // Save diff image if output path provided
    if let Some(output_path_str) = args.output.as_ref() {
        let output_path = Path::new(output_path_str);
        let out_format = ImageFormat::from_path_extension(output_path).unwrap_or(ImageFormat::Png);

        let options = EncodeOptions {
            format: out_format,
            quality: args.quality,
            strip_metadata: true,
            resolution: None,
        };

        CodecRegistry::encode(&diff_img, output_path, &options)?;
    }

    ctx.print_output(
        &format!(
            "{} vs {}: {}",
            input_a,
            input_b,
            if result.identical {
                "identical".to_string()
            } else {
                format!(
                    "{} pixels differ ({:.2}%), MAE={:.2}",
                    result.diff_pixels, result.diff_percent, result.mae
                )
            }
        ),
        &result,
    );

    if result.identical {
        Ok(0)
    } else {
        Ok(1)
    }
}
