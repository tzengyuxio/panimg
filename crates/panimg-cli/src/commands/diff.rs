use crate::app::{DiffArgs, OutputFormat};
use crate::output;
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::ops::diff;
use std::path::Path;

pub fn run(args: &DiffArgs, format: OutputFormat, dry_run: bool) -> i32 {
    let input_a = match &args.input_a {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: first input image".into(),
                suggestion: "usage: panimg diff <image_a> <image_b> [-o diff.png]".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let input_b = match &args.input_b {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: second input image".into(),
                suggestion: "usage: panimg diff <image_a> <image_b> [-o diff.png]".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let threshold = args.threshold.unwrap_or(0);

    let path_a = Path::new(input_a);
    let path_b = Path::new(input_b);

    if dry_run {
        let plan = serde_json::json!({
            "operation": "diff",
            "image_a": input_a,
            "image_b": input_b,
            "threshold": threshold,
        });
        output::print_output(
            format,
            &format!("Would compare {} vs {}", input_a, input_b),
            &plan,
        );
        return 0;
    }

    let img_a = match CodecRegistry::decode(path_a) {
        Ok(i) => i,
        Err(e) => return output::print_error(format, &e),
    };

    let img_b = match CodecRegistry::decode(path_b) {
        Ok(i) => i,
        Err(e) => return output::print_error(format, &e),
    };

    if let Err(e) = diff::validate_inputs(&img_a, &img_b) {
        return output::print_error(format, &e);
    }

    let (result, diff_img) = match diff::compare(&img_a, &img_b, threshold) {
        Ok(r) => r,
        Err(e) => return output::print_error(format, &e),
    };

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

        if let Err(e) = CodecRegistry::encode(&diff_img, output_path, &options) {
            return output::print_error(format, &e);
        }
    }

    output::print_output(
        format,
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
        0
    } else {
        1
    }
}
