use crate::app::{OutputFormat, PsdLayersArgs};
use crate::output;
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
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

pub fn run(args: &PsdLayersArgs, format: OutputFormat, dry_run: bool, schema: bool) -> i32 {
    if schema {
        let schema = serde_json::json!({
            "command": "psd-layers",
            "params": {
                "input": { "type": "string", "required": true, "description": "Input PSD file" },
                "output_dir": { "type": "string", "description": "Output directory" },
                "layer_format": { "type": "string", "default": "png", "description": "Output format" },
                "layer_index": { "type": "integer", "description": "Extract specific layer by index" },
                "layer_name": { "type": "string", "description": "Filter layers by name substring" }
            }
        });
        output::print_json(&schema);
        return 0;
    }

    let input = match &args.input {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: input".into(),
                suggestion: "usage: panimg psd-layers <input.psd> --output-dir ./layers".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let input_path = Path::new(input);

    if dry_run {
        let plan = serde_json::json!({
            "operation": "psd-layers",
            "input": input,
        });
        output::print_output(format, &format!("Would extract layers from {input}"), &plan);
        return 0;
    }

    let data = match std::fs::read(input_path) {
        Ok(d) => d,
        Err(e) => {
            let err = PanimgError::IoError {
                message: e.to_string(),
                path: Some(input_path.to_path_buf()),
                suggestion: "check that the file exists and is readable".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let output_dir = args.output_dir.as_deref().unwrap_or(".");
    let out_dir = Path::new(output_dir);
    if let Err(e) = std::fs::create_dir_all(out_dir) {
        let err = PanimgError::IoError {
            message: e.to_string(),
            path: Some(out_dir.to_path_buf()),
            suggestion: "check the output directory path".into(),
        };
        return output::print_error(format, &err);
    }

    let ext = &args.layer_format;
    let out_format = match ImageFormat::from_extension(ext) {
        Some(f) => f,
        None => {
            let err = PanimgError::InvalidArgument {
                message: format!("unsupported layer format: {ext}"),
                suggestion: "use png, jpg, webp, etc.".into(),
            };
            return output::print_error(format, &err);
        }
    };

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

    let total_layers = match panimg_core::psd::for_each_layer(&data, |info, img| {
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
    }) {
        Ok(total) => total,
        Err(e) => return output::print_error(format, &e),
    };

    if let Some(e) = encode_error {
        return output::print_error(format, &e);
    }

    let result = LayersResult {
        input: input.clone(),
        total_layers,
        extracted: layer_outputs.len(),
        output_dir: output_dir.to_string(),
        layers: layer_outputs,
    };

    output::print_output(
        format,
        &format!(
            "Extracted {}/{} layers from {} → {}",
            result.extracted, result.total_layers, result.input, result.output_dir
        ),
        &result,
    );

    0
}
