use crate::app::{BatchArgs, OutputFormat};
use crate::output;
use indicatif::{ProgressBar, ProgressStyle};
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::ops::blur::BlurOp;
use panimg_core::ops::brightness::BrightnessOp;
use panimg_core::ops::contrast::ContrastOp;
use panimg_core::ops::crop::CropOp;
use panimg_core::ops::edge_detect::EdgeDetectOp;
use panimg_core::ops::emboss::EmbossOp;
use panimg_core::ops::flip::{FlipDirection, FlipOp};
use panimg_core::ops::grayscale::GrayscaleOp;
use panimg_core::ops::hue_rotate::HueRotateOp;
use panimg_core::ops::invert::InvertOp;
use panimg_core::ops::orient::AutoOrientOp;
use panimg_core::ops::resize::{FitMode, ResizeFilter, ResizeOp};
use panimg_core::ops::rotate::{RotateAngle, RotateOp};
use panimg_core::ops::sharpen::SharpenOp;
use panimg_core::pipeline::Pipeline;
use rayon::prelude::*;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

#[derive(Serialize)]
struct BatchResult {
    total: usize,
    succeeded: usize,
    failed: usize,
    results: Vec<FileResult>,
}

#[derive(Serialize)]
struct FileResult {
    input: String,
    output: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_size: Option<u64>,
}

/// Resolve output path for a given input file.
fn resolve_output_path(
    input: &Path,
    output_dir: Option<&str>,
    output_template: Option<&str>,
    target_ext: Option<&str>,
) -> PathBuf {
    if let Some(template) = output_template {
        let stem = input.file_stem().unwrap_or_default().to_string_lossy();
        let ext = target_ext.map(|e| e.to_string()).unwrap_or_else(|| {
            input
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        });
        let dir = input.parent().unwrap_or(Path::new(".")).to_string_lossy();
        let name = input.file_name().unwrap_or_default().to_string_lossy();

        template
            .replace("{stem}", &stem)
            .replace("{name}", &name)
            .replace("{ext}", &ext)
            .replace("{dir}", &dir)
            .into()
    } else if let Some(dir) = output_dir {
        let mut filename = input
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let ext = target_ext.map(|e| e.to_string()).unwrap_or_else(|| {
            input
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        });
        filename.push('.');
        filename.push_str(&ext);
        Path::new(dir).join(filename)
    } else {
        // Fallback: same directory with _out suffix
        let stem = input.file_stem().unwrap_or_default().to_string_lossy();
        let ext = target_ext.map(|e| e.to_string()).unwrap_or_else(|| {
            input
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        });
        let dir = input.parent().unwrap_or(Path::new("."));
        dir.join(format!("{stem}_out.{ext}"))
    }
}

/// Build a pipeline from batch args for a given operation.
fn build_pipeline(args: &BatchArgs, input_path: &Path) -> Result<Pipeline, PanimgError> {
    let mut pipeline = Pipeline::new();
    let op = args.operation.as_str();

    match op {
        "convert" => {
            // No pipeline ops needed, just decode → encode with different format
        }
        "resize" => {
            let width = args.width;
            let height = args.height;
            let fit = args
                .fit
                .as_deref()
                .map(FitMode::parse)
                .transpose()?
                .unwrap_or(FitMode::Contain);
            let filter = args
                .filter
                .as_deref()
                .map(ResizeFilter::parse)
                .transpose()?
                .unwrap_or(ResizeFilter::Lanczos3);
            pipeline = pipeline.push(ResizeOp::new(width, height, fit, filter)?);
        }
        "crop" => {
            let x = args.x.unwrap_or(0);
            let y = args.y.unwrap_or(0);
            let width = args.crop_width.ok_or_else(|| PanimgError::InvalidArgument {
                message: "batch crop requires --crop-width".into(),
                suggestion: "usage: panimg batch crop '*.png' --output-dir out --crop-width 100 --crop-height 100".into(),
            })?;
            let height = args.crop_height.ok_or_else(|| PanimgError::InvalidArgument {
                message: "batch crop requires --crop-height".into(),
                suggestion: "usage: panimg batch crop '*.png' --output-dir out --crop-width 100 --crop-height 100".into(),
            })?;
            pipeline = pipeline.push(CropOp::new(x, y, width, height)?);
        }
        "rotate" => {
            let angle_str = args
                .angle
                .as_deref()
                .ok_or_else(|| PanimgError::InvalidArgument {
                    message: "batch rotate requires --angle".into(),
                    suggestion: "usage: panimg batch rotate '*.png' --output-dir out --angle 90"
                        .into(),
                })?;
            pipeline = pipeline.push(RotateOp::new(RotateAngle::parse(angle_str)?));
        }
        "flip" => {
            let dir_str =
                args.direction
                    .as_deref()
                    .ok_or_else(|| {
                        PanimgError::InvalidArgument {
                    message: "batch flip requires --direction".into(),
                    suggestion:
                        "usage: panimg batch flip '*.png' --output-dir out --direction horizontal"
                            .into(),
                }
                    })?;
            pipeline = pipeline.push(FlipOp::new(FlipDirection::parse(dir_str)?));
        }
        "auto-orient" => {
            let orient_op = AutoOrientOp::from_path(input_path);
            pipeline = pipeline.push(orient_op);
        }
        "grayscale" => {
            pipeline = pipeline.push(GrayscaleOp::new());
        }
        "invert" => {
            pipeline = pipeline.push(InvertOp::new());
        }
        "brightness" => {
            let value = args
                .brightness_value
                .ok_or_else(|| PanimgError::InvalidArgument {
                    message: "batch brightness requires --value".into(),
                    suggestion:
                        "usage: panimg batch brightness '*.png' --output-dir out --value 20".into(),
                })?;
            pipeline = pipeline.push(BrightnessOp::new(value)?);
        }
        "contrast" => {
            let value = args
                .contrast_value
                .ok_or_else(|| PanimgError::InvalidArgument {
                    message: "batch contrast requires --value".into(),
                    suggestion:
                        "usage: panimg batch contrast '*.png' --output-dir out --contrast-value 20"
                            .into(),
                })?;
            pipeline = pipeline.push(ContrastOp::new(value)?);
        }
        "hue-rotate" => {
            let degrees = args.degrees.ok_or_else(|| PanimgError::InvalidArgument {
                message: "batch hue-rotate requires --degrees".into(),
                suggestion: "usage: panimg batch hue-rotate '*.png' --output-dir out --degrees 90"
                    .into(),
            })?;
            pipeline = pipeline.push(HueRotateOp::new(degrees)?);
        }
        "blur" => {
            let sigma = args.sigma.ok_or_else(|| PanimgError::InvalidArgument {
                message: "batch blur requires --sigma".into(),
                suggestion: "usage: panimg batch blur '*.png' --output-dir out --sigma 2.0".into(),
            })?;
            pipeline = pipeline.push(BlurOp::new(sigma)?);
        }
        "sharpen" => {
            let sigma = args.sigma.ok_or_else(|| PanimgError::InvalidArgument {
                message: "batch sharpen requires --sigma".into(),
                suggestion: "usage: panimg batch sharpen '*.png' --output-dir out --sigma 1.0"
                    .into(),
            })?;
            let threshold = args.threshold.unwrap_or(0);
            pipeline = pipeline.push(SharpenOp::new(sigma, threshold)?);
        }
        "edge-detect" => {
            pipeline = pipeline.push(EdgeDetectOp::new());
        }
        "emboss" => {
            pipeline = pipeline.push(EmbossOp::new());
        }
        _ => {
            return Err(PanimgError::InvalidArgument {
                message: format!("unknown batch operation: '{op}'"),
                suggestion: "supported: convert, resize, crop, rotate, flip, auto-orient, grayscale, invert, brightness, contrast, hue-rotate, blur, sharpen, edge-detect, emboss".into(),
            });
        }
    }

    Ok(pipeline)
}

fn process_single_file(
    input_path: &Path,
    output_path: &Path,
    pipeline: &Pipeline,
    options: &EncodeOptions,
    overwrite: bool,
    skip_existing: bool,
) -> FileResult {
    let input_str = input_path.to_string_lossy().to_string();
    let output_str = output_path.to_string_lossy().to_string();

    // Check output exists
    if output_path.exists() && !overwrite {
        if skip_existing {
            return FileResult {
                input: input_str,
                output: output_str,
                status: "skipped".into(),
                error: None,
                output_size: None,
            };
        }
        return FileResult {
            input: input_str,
            output: output_str,
            status: "error".into(),
            error: Some("output already exists (use --overwrite)".into()),
            output_size: None,
        };
    }

    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return FileResult {
                    input: input_str,
                    output: output_str,
                    status: "error".into(),
                    error: Some(format!("failed to create output directory: {e}")),
                    output_size: None,
                };
            }
        }
    }

    // Decode
    let img = match CodecRegistry::decode(input_path) {
        Ok(i) => i,
        Err(e) => {
            return FileResult {
                input: input_str,
                output: output_str,
                status: "error".into(),
                error: Some(e.to_string()),
                output_size: None,
            }
        }
    };

    // Execute pipeline
    let result_img = if pipeline.is_empty() {
        img
    } else {
        match pipeline.execute(img) {
            Ok(i) => i,
            Err(e) => {
                return FileResult {
                    input: input_str,
                    output: output_str,
                    status: "error".into(),
                    error: Some(e.to_string()),
                    output_size: None,
                }
            }
        }
    };

    // Encode
    if let Err(e) = CodecRegistry::encode(&result_img, output_path, options) {
        return FileResult {
            input: input_str,
            output: output_str,
            status: "error".into(),
            error: Some(e.to_string()),
            output_size: None,
        };
    }

    let output_size = std::fs::metadata(output_path).map(|m| m.len()).ok();

    FileResult {
        input: input_str,
        output: output_str,
        status: "ok".into(),
        error: None,
        output_size,
    }
}

pub fn run(args: &BatchArgs, format: OutputFormat, dry_run: bool) -> i32 {
    // Expand glob pattern
    let files: Vec<PathBuf> = match glob::glob(&args.pattern) {
        Ok(paths) => paths
            .filter_map(|p| p.ok())
            .filter(|p| p.is_file())
            .collect(),
        Err(e) => {
            let err = PanimgError::InvalidArgument {
                message: format!("invalid glob pattern: {e}"),
                suggestion: "use a valid glob pattern like '*.png' or 'photos/**/*.jpg'".into(),
            };
            return output::print_error(format, &err);
        }
    };

    if files.is_empty() {
        let err = PanimgError::InvalidArgument {
            message: format!("no files matched pattern: '{}'", args.pattern),
            suggestion: "check the glob pattern and ensure matching files exist".into(),
        };
        return output::print_error(format, &err);
    }

    // Determine target extension for output
    let target_ext = args.to.as_deref();

    // Validate the operation can build a pipeline (check args early)
    // Use a dummy path for operations that don't depend on input path
    let dummy_path = Path::new("dummy.png");
    if args.operation != "auto-orient" {
        if let Err(e) = build_pipeline(args, dummy_path) {
            return output::print_error(format, &e);
        }
    }

    // Dry run
    if dry_run {
        let plan_files: Vec<serde_json::Value> = files
            .iter()
            .map(|f| {
                let out = resolve_output_path(
                    f,
                    args.output_dir.as_deref(),
                    args.output_template.as_deref(),
                    target_ext,
                );
                serde_json::json!({
                    "input": f.to_string_lossy(),
                    "output": out.to_string_lossy(),
                })
            })
            .collect();

        let plan = serde_json::json!({
            "operation": args.operation,
            "total_files": files.len(),
            "files": plan_files,
        });

        output::print_output(
            format,
            &format!("Would {} {} files", args.operation, files.len()),
            &plan,
        );
        return 0;
    }

    // Setup progress bar (only for human output)
    let pb = match format {
        OutputFormat::Human => {
            let pb = ProgressBar::new(files.len() as u64);
            pb.set_style(
                ProgressStyle::with_template(
                    "{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}",
                )
                .unwrap()
                .progress_chars("█▓░"),
            );
            Some(pb)
        }
        OutputFormat::Json => None,
    };

    let succeeded = AtomicUsize::new(0);
    let failed = AtomicUsize::new(0);
    let results = Mutex::new(Vec::with_capacity(files.len()));

    // Process files in parallel
    files.par_iter().for_each(|input_path| {
        let output_path = resolve_output_path(
            input_path,
            args.output_dir.as_deref(),
            args.output_template.as_deref(),
            target_ext,
        );

        // Build pipeline per-file (needed for auto-orient which reads EXIF per file)
        let pipeline = match build_pipeline(args, input_path) {
            Ok(p) => p,
            Err(e) => {
                let file_result = FileResult {
                    input: input_path.to_string_lossy().to_string(),
                    output: output_path.to_string_lossy().to_string(),
                    status: "error".into(),
                    error: Some(e.to_string()),
                    output_size: None,
                };
                failed.fetch_add(1, Ordering::Relaxed);
                results.lock().unwrap().push(file_result);
                if let Some(ref pb) = pb {
                    pb.inc(1);
                }
                return;
            }
        };

        let out_format = ImageFormat::from_path_extension(&output_path)
            .or_else(|| args.to.as_deref().and_then(ImageFormat::from_extension))
            .or_else(|| ImageFormat::from_path_extension(input_path))
            .unwrap_or(ImageFormat::Png);

        let options = EncodeOptions {
            format: out_format,
            quality: args.quality,
            strip_metadata: args.strip,
        };

        let file_result = process_single_file(
            input_path,
            &output_path,
            &pipeline,
            &options,
            args.overwrite,
            args.skip_existing,
        );

        match file_result.status.as_str() {
            "ok" => {
                succeeded.fetch_add(1, Ordering::Relaxed);
            }
            "skipped" => {
                // don't count as success or failure
            }
            _ => {
                failed.fetch_add(1, Ordering::Relaxed);
            }
        }

        if let Some(ref pb) = pb {
            pb.set_message(
                input_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            );
            pb.inc(1);
        }

        results.lock().unwrap().push(file_result);
    });

    if let Some(ref pb) = pb {
        pb.finish_and_clear();
    }

    let succeeded_count = succeeded.load(Ordering::Relaxed);
    let failed_count = failed.load(Ordering::Relaxed);
    let file_results = results.into_inner().unwrap();

    let batch_result = BatchResult {
        total: files.len(),
        succeeded: succeeded_count,
        failed: failed_count,
        results: file_results,
    };

    output::print_output(
        format,
        &format!(
            "Batch {}: {}/{} succeeded, {} failed",
            args.operation,
            succeeded_count,
            files.len(),
            failed_count
        ),
        &batch_result,
    );

    if failed_count > 0 {
        // Print failed files in human mode
        if matches!(format, OutputFormat::Human) {
            for r in &batch_result.results {
                if r.status == "error" {
                    eprintln!(
                        "  error: {} — {}",
                        r.input,
                        r.error.as_deref().unwrap_or("unknown")
                    );
                }
            }
        }
        1
    } else {
        0
    }
}
