use crate::app::{BlurArgs, RunContext};
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::ops::blur::{BilateralBlurOp, BlurOp, BoxBlurOp, MedianBlurOp, MotionBlurOp};
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct BlurResult {
    input: String,
    output: String,
    method: String,
    params: serde_json::Value,
    output_size: u64,
}

pub fn run(args: &BlurArgs, ctx: &RunContext) -> i32 {
    if ctx.schema {
        let s = BlurOp::schema();
        ctx.print_json(&serde_json::to_value(&s).unwrap());
        return 0;
    }

    let input = match &args.input {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: input".into(),
                suggestion: "usage: panimg blur <input> -o <output> --sigma 2.0".into(),
            };
            return ctx.print_error(&err);
        }
    };

    let output_path_str = match args.output.as_ref().or(args.output_pos.as_ref()) {
        Some(o) => o.clone(),
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: output (-o)".into(),
                suggestion: "usage: panimg blur <input> -o <output> --sigma 2.0".into(),
            };
            return ctx.print_error(&err);
        }
    };

    let method = args.method.as_str();
    let pipeline = match build_blur_pipeline(args, method) {
        Ok(p) => p,
        Err(e) => return ctx.print_error(&e),
    };

    let input_path = Path::new(input);
    let output_path = Path::new(&output_path_str);

    if ctx.dry_run {
        let plan = pipeline.describe();
        ctx.print_output(
            &format!(
                "Would apply {method} blur to {} → {}",
                input, output_path_str
            ),
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
    let desc = pipeline.describe();
    let params = desc
        .steps
        .first()
        .map(|s| s.params.clone())
        .unwrap_or_default();

    let result = BlurResult {
        input: input.clone(),
        output: output_path_str,
        method: method.to_string(),
        params,
        output_size,
    };

    ctx.print_output(
        &desc
            .steps
            .first()
            .map(|s| s.description.clone())
            .unwrap_or_else(|| format!("{method} blur applied")),
        &result,
    );

    0
}

fn build_blur_pipeline(
    args: &BlurArgs,
    method: &str,
) -> std::result::Result<Pipeline, PanimgError> {
    match method {
        "gaussian" => {
            let sigma = args.sigma.ok_or_else(|| PanimgError::InvalidArgument {
                message: "gaussian blur requires --sigma".into(),
                suggestion: "usage: panimg blur <input> -o <output> --sigma 2.0".into(),
            })?;
            Ok(Pipeline::new().push(BlurOp::new(sigma)?))
        }
        "box" => {
            let radius = args.radius.ok_or_else(|| PanimgError::InvalidArgument {
                message: "box blur requires --radius".into(),
                suggestion: "usage: panimg blur <input> -o <output> --method box --radius 3".into(),
            })?;
            Ok(Pipeline::new().push(BoxBlurOp::new(radius)?))
        }
        "motion" => {
            let angle = args.angle.unwrap_or(0.0);
            let distance = args.distance.ok_or_else(|| PanimgError::InvalidArgument {
                message: "motion blur requires --distance".into(),
                suggestion:
                    "usage: panimg blur <input> -o <output> --method motion --distance 10 --angle 45"
                        .into(),
            })?;
            Ok(Pipeline::new().push(MotionBlurOp::new(angle, distance)?))
        }
        "median" => {
            let radius = args.radius.ok_or_else(|| PanimgError::InvalidArgument {
                message: "median blur requires --radius".into(),
                suggestion: "usage: panimg blur <input> -o <output> --method median --radius 2"
                    .into(),
            })?;
            Ok(Pipeline::new().push(MedianBlurOp::new(radius)?))
        }
        "bilateral" => {
            let sigma = args.sigma.ok_or_else(|| PanimgError::InvalidArgument {
                message: "bilateral blur requires --sigma (spatial sigma)".into(),
                suggestion: "usage: panimg blur <input> -o <output> --method bilateral --sigma 5.0 --sigma-color 50 --radius 5".into(),
            })?;
            let sigma_color = args.sigma_color.ok_or_else(|| PanimgError::InvalidArgument {
                message: "bilateral blur requires --sigma-color".into(),
                suggestion: "usage: panimg blur <input> -o <output> --method bilateral --sigma 5.0 --sigma-color 50 --radius 5".into(),
            })?;
            let radius = args.radius.ok_or_else(|| PanimgError::InvalidArgument {
                message: "bilateral blur requires --radius".into(),
                suggestion: "usage: panimg blur <input> -o <output> --method bilateral --sigma 5.0 --sigma-color 50 --radius 5".into(),
            })?;
            Ok(Pipeline::new().push(BilateralBlurOp::new(radius, sigma, sigma_color)?))
        }
        _ => Err(PanimgError::InvalidArgument {
            message: format!("unknown blur method: '{method}'"),
            suggestion: "supported methods: gaussian, box, motion, median, bilateral".into(),
        }),
    }
}
