use crate::app::{RunContext, SetDensityArgs};
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::ops::resize::{FitMode, ResizeFilter, ResizeOp};
use panimg_core::ops::Operation;
use panimg_core::resolution::{read_resolution, Resolution, ResolutionUnit};
use panimg_core::schema::{CommandSchema, ParamRange, ParamSchema, ParamType};
use serde::Serialize;
use std::path::Path;

pub fn schema() -> CommandSchema {
    CommandSchema {
        command: "set-density".into(),
        description:
            "Set image resolution/density (DPI/DPCM) metadata, optionally resampling pixels".into(),
        params: vec![
            ParamSchema {
                name: "input".into(),
                param_type: ParamType::Path,
                required: true,
                description: "Input image file path".into(),
                default: None,
                choices: None,
                range: None,
            },
            ParamSchema {
                name: "output".into(),
                param_type: ParamType::Path,
                required: true,
                description: "Output file path".into(),
                default: None,
                choices: None,
                range: None,
            },
            ParamSchema {
                name: "density".into(),
                param_type: ParamType::Float,
                required: true,
                description: "Target density value".into(),
                default: None,
                choices: None,
                range: Some(ParamRange {
                    min: 0.1,
                    max: 100000.0,
                }),
            },
            ParamSchema {
                name: "unit".into(),
                param_type: ParamType::String,
                required: false,
                description: "Density unit".into(),
                default: Some(serde_json::json!("dpi")),
                choices: Some(vec!["dpi".into(), "dpcm".into()]),
                range: None,
            },
            ParamSchema {
                name: "resample".into(),
                param_type: ParamType::Boolean,
                required: false,
                description: "Resample pixels to maintain physical print size".into(),
                default: Some(serde_json::json!(false)),
                choices: None,
                range: None,
            },
            ParamSchema {
                name: "source-density".into(),
                param_type: ParamType::Float,
                required: false,
                description: "Source density (overrides EXIF, used with --resample)".into(),
                default: None,
                choices: None,
                range: Some(ParamRange {
                    min: 0.1,
                    max: 100000.0,
                }),
            },
            ParamSchema {
                name: "source-unit".into(),
                param_type: ParamType::String,
                required: false,
                description: "Source density unit".into(),
                default: Some(serde_json::json!("dpi")),
                choices: Some(vec!["dpi".into(), "dpcm".into()]),
                range: None,
            },
            ParamSchema {
                name: "filter".into(),
                param_type: ParamType::String,
                required: false,
                description: "Resize filter for resampling".into(),
                default: Some(serde_json::json!("lanczos3")),
                choices: Some(vec![
                    "lanczos3".into(),
                    "catmull-rom".into(),
                    "nearest".into(),
                    "linear".into(),
                ]),
                range: None,
            },
            ParamSchema {
                name: "quality".into(),
                param_type: ParamType::Integer,
                required: false,
                description: "Output quality (1-100, for lossy formats)".into(),
                default: None,
                choices: None,
                range: Some(ParamRange {
                    min: 1.0,
                    max: 100.0,
                }),
            },
            ParamSchema {
                name: "strip".into(),
                param_type: ParamType::Boolean,
                required: false,
                description: "Strip metadata from output".into(),
                default: Some(serde_json::json!(false)),
                choices: None,
                range: None,
            },
        ],
    }
}

#[derive(Serialize)]
struct SetDensityPlan {
    input: String,
    output: String,
    density: f64,
    unit: String,
    resample: bool,
    source_density: Option<f64>,
    source_unit: String,
    filter: String,
}

#[derive(Serialize)]
struct SetDensityResult {
    input: String,
    output: String,
    density: f64,
    unit: String,
    resampled: bool,
    input_size: u64,
    output_size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    original_dimensions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_dimensions: Option<String>,
}

pub fn run(args: &SetDensityArgs, ctx: &RunContext) -> i32 {
    if ctx.schema {
        let s = schema();
        ctx.print_json(&serde_json::to_value(&s).unwrap());
        return 0;
    }

    let input = match &args.input {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: input".into(),
                suggestion: "usage: panimg set-density <input> -o <output> --density <value>"
                    .into(),
            };
            return ctx.print_error(&err);
        }
    };

    let output_path_str = match args.output.as_ref().or(args.output_pos.as_ref()) {
        Some(o) => o.clone(),
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: output (-o)".into(),
                suggestion: "usage: panimg set-density <input> -o <output> --density <value>"
                    .into(),
            };
            return ctx.print_error(&err);
        }
    };

    let input_path = Path::new(input);
    let output_path = Path::new(&output_path_str);

    // Parse target unit and resolution
    let target_unit = match ResolutionUnit::parse(&args.unit) {
        Ok(u) => u,
        Err(e) => return ctx.print_error(&e),
    };
    let target_res = Resolution::from_density(args.density, target_unit);

    // Determine output format
    let out_format = ImageFormat::from_path_extension(output_path)
        .unwrap_or(ImageFormat::from_path(input_path).unwrap_or(ImageFormat::Png));

    // Dry run
    if ctx.dry_run {
        let plan = SetDensityPlan {
            input: input.clone(),
            output: output_path_str,
            density: args.density,
            unit: args.unit.clone(),
            resample: args.resample,
            source_density: args.source_density,
            source_unit: args.source_unit.clone(),
            filter: args.filter.clone(),
        };
        let resample_msg = if args.resample {
            " (with resampling)"
        } else {
            ""
        };
        ctx.print_output(
            &format!(
                "Would set density of {} to {} {}{} → {}",
                input, args.density, args.unit, resample_msg, plan.output
            ),
            &plan,
        );
        return 0;
    }

    // Decode
    let decode_opts = ctx.decode_options();
    let mut img = match CodecRegistry::decode_with_options(input_path, &decode_opts) {
        Ok(i) => i,
        Err(e) => return ctx.print_error(&e),
    };

    let input_size = std::fs::metadata(input_path).map(|m| m.len()).unwrap_or(0);
    let orig_w = img.width();
    let orig_h = img.height();

    // Resample if requested
    if args.resample {
        let source_unit = match ResolutionUnit::parse(&args.source_unit) {
            Ok(u) => u,
            Err(e) => return ctx.print_error(&e),
        };
        let filter = match ResizeFilter::parse(&args.filter) {
            Ok(f) => f,
            Err(e) => return ctx.print_error(&e),
        };

        // Determine source resolution
        let source_res = if let Some(src_density) = args.source_density {
            if src_density <= 0.0 {
                let err = PanimgError::InvalidArgument {
                    message: format!("--source-density must be positive, got {src_density}"),
                    suggestion: "specify a positive density value".into(),
                };
                return ctx.print_error(&err);
            }
            Resolution::from_density(src_density, source_unit)
        } else {
            match read_resolution(input_path) {
                Some(r) => r,
                None => {
                    let err = PanimgError::InvalidArgument {
                        message: "cannot resample: no resolution metadata found in input".into(),
                        suggestion: "specify --source-density and --source-unit manually".into(),
                    };
                    return ctx.print_error(&err);
                }
            }
        };

        // Calculate scale factor: target_dpi / source_dpi
        let scale_x = target_res.x_dpi / source_res.x_dpi;
        let scale_y = target_res.y_dpi / source_res.y_dpi;

        let new_w = (orig_w as f64 * scale_x).round() as u32;
        let new_h = (orig_h as f64 * scale_y).round() as u32;

        if new_w == 0 || new_h == 0 {
            let err = PanimgError::InvalidArgument {
                message: format!("resampled dimensions would be zero: {new_w}x{new_h}"),
                suggestion: "check that density values produce reasonable scaling".into(),
            };
            return ctx.print_error(&err);
        }

        if new_w != orig_w || new_h != orig_h {
            let resize_op = match ResizeOp::new(Some(new_w), Some(new_h), FitMode::Fill, filter) {
                Ok(op) => op,
                Err(e) => return ctx.print_error(&e),
            };

            img = match resize_op.apply(img) {
                Ok(i) => i,
                Err(e) => return ctx.print_error(&e),
            };
        }
    }

    let final_w = img.width();
    let final_h = img.height();

    // Encode with resolution metadata
    let options = EncodeOptions {
        format: out_format,
        quality: args.quality,
        strip_metadata: args.strip,
        resolution: Some(target_res),
    };

    if let Err(e) = CodecRegistry::encode(&img, output_path, &options) {
        return ctx.print_error(&e);
    }

    let output_size = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);

    let result = SetDensityResult {
        input: input.clone(),
        output: output_path_str,
        density: args.density,
        unit: args.unit.clone(),
        resampled: args.resample,
        input_size,
        output_size,
        original_dimensions: if args.resample {
            Some(format!("{orig_w}x{orig_h}"))
        } else {
            None
        },
        output_dimensions: if args.resample {
            Some(format!("{final_w}x{final_h}"))
        } else {
            None
        },
    };

    let human_msg = if args.resample {
        format!(
            "Set density {} {} on {} → {} (resampled {orig_w}x{orig_h} → {final_w}x{final_h})",
            args.density, args.unit, result.input, result.output
        )
    } else {
        format!(
            "Set density {} {} on {} → {}",
            args.density, args.unit, result.input, result.output
        )
    };

    ctx.print_output(&human_msg, &result);
    0
}
