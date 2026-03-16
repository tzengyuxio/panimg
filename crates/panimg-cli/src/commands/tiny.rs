use crate::app::{OutputFormat, RunContext, TinyArgs};
use panimg_core::compress::{compress, CompressOptions};
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::schema::{CommandSchema, ParamRange, ParamSchema, ParamType};
use serde::Serialize;
use std::path::Path;

pub fn schema() -> CommandSchema {
    CommandSchema {
        command: "tiny".into(),
        description: "Smart image compression (like TinyPNG)".into(),
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
                required: false,
                description: "Output file path (default: {stem}_tiny.{ext})".into(),
                default: None,
                choices: None,
                range: None,
            },
            ParamSchema {
                name: "quality".into(),
                param_type: ParamType::Integer,
                required: false,
                description: "Compression quality (1-100)".into(),
                default: None,
                choices: None,
                range: Some(ParamRange {
                    min: 1.0,
                    max: 100.0,
                }),
            },
            ParamSchema {
                name: "max-colors".into(),
                param_type: ParamType::Integer,
                required: false,
                description: "PNG: max colors for quantization (2-256)".into(),
                default: Some(serde_json::json!(256)),
                choices: None,
                range: Some(ParamRange {
                    min: 2.0,
                    max: 256.0,
                }),
            },
            ParamSchema {
                name: "lossless".into(),
                param_type: ParamType::Boolean,
                required: false,
                description: "PNG: skip quantization, only lossless optimization".into(),
                default: Some(serde_json::json!(false)),
                choices: None,
                range: None,
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
struct TinyPlan {
    input: String,
    output: String,
    format: String,
    quality: Option<u8>,
    max_colors: u16,
    lossless: bool,
    strip_metadata: bool,
}

#[derive(Serialize)]
struct TinyResult {
    input: String,
    output: String,
    format: String,
    input_size: u64,
    output_size: u64,
    savings_percent: f64,
}

pub fn run(args: &TinyArgs, ctx: &RunContext) -> i32 {
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
                suggestion: "usage: panimg tiny <input> [-o <output>]".into(),
            };
            return ctx.print_error(&err);
        }
    };

    let input_path = Path::new(input);

    // Resolve output path
    let output_path_str = if let Some(o) = args.output.as_ref().or(args.output_pos.as_ref()) {
        o.clone()
    } else {
        let stem = input_path.file_stem().unwrap_or_default().to_string_lossy();
        let ext = input_path.extension().unwrap_or_default().to_string_lossy();
        let dir = input_path.parent().unwrap_or(Path::new("."));
        dir.join(format!("{stem}_tiny.{ext}"))
            .to_string_lossy()
            .to_string()
    };

    let output_path = Path::new(&output_path_str);

    // Check output exists
    if output_path.exists() && !args.overwrite {
        if args.skip_existing {
            match ctx.format {
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
        return ctx.print_error(&err);
    }

    // Dry run — need format detection here only for display
    if ctx.dry_run {
        let img_format = ImageFormat::from_path(input_path)
            .map(|f| f.to_string())
            .unwrap_or_else(|| "unknown".into());
        let plan = TinyPlan {
            input: input.clone(),
            output: output_path_str,
            format: img_format.clone(),
            quality: args.quality,
            max_colors: args.max_colors,
            lossless: args.lossless,
            strip_metadata: args.strip,
        };
        ctx.print_output(&format!("Would compress {} ({})", input, img_format), &plan);
        return 0;
    }

    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                let err = PanimgError::IoError {
                    message: e.to_string(),
                    path: Some(parent.to_path_buf()),
                    suggestion: "check output directory permissions".into(),
                };
                return ctx.print_error(&err);
            }
        }
    }

    let options = CompressOptions {
        quality: args.quality,
        max_colors: args.max_colors,
        lossless: args.lossless,
        strip_metadata: args.strip,
    };

    match compress(input_path, output_path, &options) {
        Ok(result) => {
            let tiny_result = TinyResult {
                input: input.clone(),
                output: output_path_str,
                format: result.format.clone(),
                input_size: result.input_size,
                output_size: result.output_size,
                savings_percent: result.savings_percent,
            };

            let human_msg = if result.savings_percent > 0.0 {
                format!(
                    "Compressed {} → {} ({} → {}, {:.1}% smaller)",
                    tiny_result.input,
                    tiny_result.output,
                    format_size(result.input_size),
                    format_size(result.output_size),
                    result.savings_percent,
                )
            } else {
                format!(
                    "Compressed {} → {} ({} → {}, {:.1}% larger — already optimized?)",
                    tiny_result.input,
                    tiny_result.output,
                    format_size(result.input_size),
                    format_size(result.output_size),
                    -result.savings_percent,
                )
            };

            ctx.print_output(&human_msg, &tiny_result);
            0
        }
        Err(e) => ctx.print_error(&e),
    }
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
