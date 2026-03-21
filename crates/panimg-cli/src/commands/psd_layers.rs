use super::common::require_input;
use super::CommandResult;
use crate::app::{PsdLayersArgs, RunContext};
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::schema::{CommandSchema, ParamSchema, ParamType};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct LayersResult {
    input: String,
    total_layers: usize,
    extracted: usize,
    output_dir: String,
    layers: Vec<LayerOutput>,
}

#[derive(Serialize)]
struct LayerOutput {
    index: usize,
    name: String,
    width: u32,
    height: u32,
    path: String,
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

pub fn schema() -> CommandSchema {
    CommandSchema {
        command: "psd-layers".into(),
        description: "Extract individual layers from a PSD file".into(),
        params: vec![
            ParamSchema {
                name: "input".into(),
                param_type: ParamType::Path,
                required: true,
                description: "Input PSD file".into(),
                default: None,
                choices: None,
                range: None,
            },
            ParamSchema {
                name: "output_dir".into(),
                param_type: ParamType::Path,
                required: false,
                description: "Output directory".into(),
                default: Some(serde_json::json!(".")),
                choices: None,
                range: None,
            },
            ParamSchema {
                name: "layer_format".into(),
                param_type: ParamType::String,
                required: false,
                description: "Output format".into(),
                default: Some(serde_json::json!("png")),
                choices: None,
                range: None,
            },
            ParamSchema {
                name: "layer_index".into(),
                param_type: ParamType::Integer,
                required: false,
                description: "Extract specific layer by index".into(),
                default: None,
                choices: None,
                range: None,
            },
            ParamSchema {
                name: "layer_name".into(),
                param_type: ParamType::String,
                required: false,
                description: "Filter layers by name substring".into(),
                default: None,
                choices: None,
                range: None,
            },
        ],
    }
}

pub fn run(args: &PsdLayersArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(
        &args.input,
        "panimg psd-layers <input.psd> --output-dir ./layers",
    )?;

    let input_path = Path::new(input);

    if ctx.dry_run {
        let plan = serde_json::json!({
            "operation": "psd-layers",
            "input": input,
        });
        ctx.print_output(&format!("Would extract layers from {input}"), &plan);
        return Ok(0);
    }

    let data = std::fs::read(input_path).map_err(|e| PanimgError::IoError {
        message: e.to_string(),
        path: Some(input_path.to_path_buf()),
        suggestion: "check that the file exists and is readable".into(),
    })?;

    let output_dir = args.output_dir.as_deref().unwrap_or(".");
    let out_dir = Path::new(output_dir);
    std::fs::create_dir_all(out_dir).map_err(|e| PanimgError::IoError {
        message: e.to_string(),
        path: Some(out_dir.to_path_buf()),
        suggestion: "check the output directory path".into(),
    })?;

    let ext = &args.layer_format;
    let out_format =
        ImageFormat::from_extension(ext).ok_or_else(|| PanimgError::InvalidArgument {
            message: format!("unsupported layer format: {ext}"),
            suggestion: "use png, jpg, webp, etc.".into(),
        })?;

    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("psd");

    let encode_opts = EncodeOptions {
        format: out_format,
        quality: None,
        strip_metadata: false,
        resolution: None,
    };

    let filter_index = args.layer_index;
    let filter_name = args.layer_name.clone();
    let mut layer_outputs = Vec::new();
    let mut encode_error: Option<PanimgError> = None;

    let total_layers = panimg_core::psd::for_each_layer(&data, |info, img| {
        // Filter by index
        if let Some(idx) = filter_index {
            if info.index != idx {
                return Ok(true);
            }
        }
        // Filter by name
        if let Some(ref name_filter) = filter_name {
            if !info.name.contains(name_filter.as_str()) {
                return Ok(true);
            }
        }

        let sanitized = sanitize_name(&info.name);
        let filename = format!("{stem}_layer_{:03}_{sanitized}.{ext}", info.index);
        let layer_path = out_dir.join(&filename);

        if let Err(e) = CodecRegistry::encode(&img, &layer_path, &encode_opts) {
            encode_error = Some(e);
            return Ok(false);
        }

        layer_outputs.push(LayerOutput {
            index: info.index,
            name: info.name,
            width: info.width,
            height: info.height,
            path: layer_path.to_string_lossy().to_string(),
        });

        // Stop early if we found the specific layer index
        if filter_index.is_some() {
            return Ok(false);
        }
        Ok(true)
    })?;

    if let Some(e) = encode_error {
        return Err(e);
    }

    let result = LayersResult {
        input: input.to_string(),
        total_layers,
        extracted: layer_outputs.len(),
        output_dir: output_dir.to_string(),
        layers: layer_outputs,
    };

    ctx.print_output(
        &format!(
            "Extracted {}/{} layers from {} → {}",
            result.extracted, result.total_layers, result.input, result.output_dir
        ),
        &result,
    );

    Ok(0)
}
