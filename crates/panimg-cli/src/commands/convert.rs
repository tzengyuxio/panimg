use crate::app::{ConvertArgs, OutputFormat};
use crate::output;
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
}

#[derive(Serialize)]
struct ConvertResult {
    input: String,
    output: String,
    from_format: String,
    to_format: String,
    input_size: u64,
    output_size: u64,
}

pub fn run(args: &ConvertArgs, format: OutputFormat, dry_run: bool, show_schema: bool) -> i32 {
    if show_schema {
        let s = schema();
        output::print_json(&serde_json::to_value(&s).unwrap());
        return 0;
    }

    let input = match &args.input {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: input".into(),
                suggestion: "usage: panimg convert <input> -o <output>".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let output_path_str = match args.output.as_ref().or(args.output_pos.as_ref()) {
        Some(o) => o.clone(),
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: output (-o)".into(),
                suggestion: "usage: panimg convert <input> -o <output>".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let input_path = Path::new(input);
    let output_path = Path::new(&output_path_str);

    // Determine target format
    let target_format = if let Some(to) = &args.to {
        match ImageFormat::from_extension(to) {
            Some(f) => f,
            None => {
                let err = PanimgError::UnsupportedFormat {
                    format: to.clone(),
                    suggestion: "use a supported format: jpg, png, webp, gif, bmp, tiff, qoi"
                        .into(),
                };
                return output::print_error(format, &err);
            }
        }
    } else {
        match ImageFormat::from_path_extension(output_path) {
            Some(f) => f,
            None => {
                let err = PanimgError::UnknownFormat {
                    path: output_path.to_path_buf(),
                    suggestion: "specify --to <format> or use a recognized output extension".into(),
                };
                return output::print_error(format, &err);
            }
        }
    };

    // Check output exists
    if output_path.exists() && !args.overwrite {
        if args.skip_existing {
            match format {
                OutputFormat::Human => println!("Skipped: output already exists"),
                OutputFormat::Json => {
                    println!(r#"{{"status": "skipped", "reason": "output_exists"}}"#)
                }
            }
            return 0;
        }
        let err = PanimgError::OutputExists {
            path: output_path.to_path_buf(),
            suggestion: "use --overwrite to replace or --skip-existing to skip".into(),
        };
        return output::print_error(format, &err);
    }

    // Detect input format
    let input_format = match ImageFormat::from_path(input_path) {
        Some(f) => f,
        None => {
            let err = PanimgError::UnknownFormat {
                path: input_path.to_path_buf(),
                suggestion: "the input file format could not be detected".into(),
            };
            return output::print_error(format, &err);
        }
    };

    // Dry run
    if dry_run {
        let plan = ConvertPlan {
            input: input.clone(),
            output: output_path_str,
            from_format: input_format.to_string(),
            to_format: target_format.to_string(),
            quality: args.quality,
            strip_metadata: args.strip,
        };
        output::print_output(
            format,
            &format!(
                "Would convert {} ({}) → {} ({})",
                input, input_format, plan.output, target_format
            ),
            &plan,
        );
        return 0;
    }

    // Decode
    let img = match CodecRegistry::decode(input_path) {
        Ok(i) => i,
        Err(e) => return output::print_error(format, &e),
    };

    let input_size = std::fs::metadata(input_path).map(|m| m.len()).unwrap_or(0);

    // Encode
    let options = EncodeOptions {
        format: target_format,
        quality: args.quality,
        strip_metadata: args.strip,
    };

    if let Err(e) = CodecRegistry::encode(&img, output_path, &options) {
        return output::print_error(format, &e);
    }

    let output_size = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);

    let result = ConvertResult {
        input: input.clone(),
        output: output_path_str,
        from_format: input_format.to_string(),
        to_format: target_format.to_string(),
        input_size,
        output_size,
    };

    output::print_output(
        format,
        &format!(
            "Converted {} → {} ({} → {})",
            result.input, result.output, result.from_format, result.to_format
        ),
        &result,
    );

    0
}
