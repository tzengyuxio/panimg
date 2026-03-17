use super::common::{require_input, require_output};
use super::CommandResult;
use crate::app::{ConvertArgs, OutputFormat, RunContext};
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::schema::{CommandSchema, ParamRange, ParamSchema, ParamType};
use serde::Serialize;
use std::path::Path;

pub fn schema() -> CommandSchema {
    CommandSchema {
        command: "convert".into(),
        description: "Convert image between formats".into(),
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
                name: "to".into(),
                param_type: ParamType::String,
                required: false,
                description: "Target format (inferred from output extension if not set)".into(),
                default: None,
                choices: Some(
                    ImageFormat::all()
                        .iter()
                        .filter(|f| f.can_encode())
                        .map(|f| f.extension().to_string())
                        .collect(),
                ),
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
            ParamSchema {
                name: "overwrite".into(),
                param_type: ParamType::Boolean,
                required: false,
                description: "Overwrite output if it exists".into(),
                default: Some(serde_json::json!(false)),
                choices: None,
                range: None,
            },
            ParamSchema {
                name: "skip-existing".into(),
                param_type: ParamType::Boolean,
                required: false,
                description: "Skip if output already exists".into(),
                default: Some(serde_json::json!(false)),
                choices: None,
                range: None,
            },
            ParamSchema {
                name: "page".into(),
                param_type: ParamType::Integer,
                required: false,
                description: "PDF page to convert (1-based, default: 1)".into(),
                default: Some(serde_json::json!(1)),
                choices: None,
                range: Some(ParamRange {
                    min: 1.0,
                    max: 10000.0,
                }),
            },
        ],
    }
}

#[derive(Serialize)]
struct ConvertPlan {
    input: String,
    output: String,
    from_format: String,
    to_format: String,
    quality: Option<u8>,
    strip_metadata: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<usize>,
}

#[derive(Serialize)]
struct ConvertResult {
    input: String,
    output: String,
    from_format: String,
    to_format: String,
    input_size: u64,
    output_size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<usize>,
}

pub fn run(args: &ConvertArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(&args.input, "panimg convert <input> -o <output>")?;
    let output_path_str = require_output(
        &args.output,
        &args.output_pos,
        "panimg convert <input> -o <output>",
    )?;

    let input_path = Path::new(input);
    let output_path = Path::new(&output_path_str);

    // Determine target format
    let target_format = if let Some(to) = &args.to {
        ImageFormat::from_extension(to).ok_or_else(|| PanimgError::UnsupportedFormat {
            format: to.clone(),
            suggestion: "use a supported format: jpg, png, webp, gif, bmp, tiff, qoi".into(),
        })?
    } else {
        ImageFormat::from_path_extension(output_path).ok_or_else(|| PanimgError::UnknownFormat {
            path: output_path.to_path_buf(),
            suggestion: "specify --to <format> or use a recognized output extension".into(),
        })?
    };

    // Check output exists
    if output_path.exists() && !args.overwrite {
        if args.skip_existing {
            match ctx.format {
                OutputFormat::Human => println!("Skipped: output already exists"),
                OutputFormat::Json => {
                    println!(r#"{{"status": "skipped", "reason": "output_exists"}}"#)
                }
            }
            return Ok(0);
        }
        return Err(PanimgError::OutputExists {
            path: output_path.to_path_buf(),
            suggestion: "use --overwrite to replace or --skip-existing to skip".into(),
        });
    }

    // Validate --page early (before dry-run and decode)
    if let Some(0) = args.page {
        return Err(PanimgError::InvalidArgument {
            message: "page numbers are 1-based, 0 is not valid".into(),
            suggestion: "use --page 1 for the first page".into(),
        });
    }

    // Detect input format
    let input_format =
        ImageFormat::from_path(input_path).ok_or_else(|| PanimgError::UnknownFormat {
            path: input_path.to_path_buf(),
            suggestion: "the input file format could not be detected".into(),
        })?;

    // Dry run
    if ctx.dry_run {
        let plan = ConvertPlan {
            input: input.to_string(),
            output: output_path_str,
            from_format: input_format.to_string(),
            to_format: target_format.to_string(),
            quality: args.quality,
            strip_metadata: args.strip,
            page: args.page,
        };
        ctx.print_output(
            &format!(
                "Would convert {} ({}) → {} ({})",
                input, input_format, plan.output, target_format
            ),
            &plan,
        );
        return Ok(0);
    }

    // Decode
    let mut decode_opts = ctx.decode_options();
    decode_opts.page = args.page.map(|p| p.saturating_sub(1));
    let mut img = CodecRegistry::decode_with_options(input_path, &decode_opts)?;

    let input_size = std::fs::metadata(input_path).map(|m| m.len()).unwrap_or(0);

    // Apply ICC color space conversion if requested
    #[cfg(feature = "icc")]
    {
        if let Some(ref target_cs) = args.convert_profile {
            let color_space = target_cs
                .parse::<panimg_core::icc::ColorSpace>()
                .map_err(|_| PanimgError::InvalidArgument {
                    message: format!("unknown color space: '{target_cs}'"),
                    suggestion: "use one of: srgb, adobe-rgb, display-p3".into(),
                })?;

            // Try to extract source ICC profile from the input file
            let input_data = std::fs::read(input_path).ok();
            let source_icc = input_data
                .as_deref()
                .and_then(panimg_core::icc::extract_icc_from_image);

            img =
                panimg_core::icc::convert_to_color_space(&img, source_icc.as_deref(), color_space)?;
        }
    }

    // Encode
    let options = EncodeOptions {
        format: target_format,
        quality: args.quality,
        strip_metadata: args.strip,
        resolution: None,
    };

    CodecRegistry::encode(&img, output_path, &options)?;

    let output_size = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);

    let result = ConvertResult {
        input: input.to_string(),
        output: output_path_str,
        from_format: input_format.to_string(),
        to_format: target_format.to_string(),
        input_size,
        output_size,
        page: args.page,
    };

    ctx.print_output(
        &format!(
            "Converted {} → {} ({} → {})",
            result.input, result.output, result.from_format, result.to_format
        ),
        &result,
    );

    Ok(0)
}
