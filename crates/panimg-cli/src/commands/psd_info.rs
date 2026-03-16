use crate::app::{OutputFormat, PsdInfoArgs};
use crate::output;
use panimg_core::error::PanimgError;
use std::path::Path;

pub fn run(args: &PsdInfoArgs, format: OutputFormat, schema: bool) -> i32 {
    if schema {
        let schema = serde_json::json!({
            "command": "psd-info",
            "params": {
                "input": { "type": "string", "required": true, "description": "Input PSD file" }
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
                suggestion: "usage: panimg psd-info <input.psd>".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let path = Path::new(input);
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            let err = PanimgError::IoError {
                message: e.to_string(),
                path: Some(path.to_path_buf()),
                suggestion: "check that the file exists and is readable".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let info = match panimg_core::psd::get_psd_info(&data) {
        Ok(i) => i,
        Err(e) => return output::print_error(format, &e),
    };

    let human = format!(
        "PSD: {}x{}, {} layer(s)\n{}",
        info.width,
        info.height,
        info.layers.len(),
        info.layers
            .iter()
            .map(|l| format!("  [{}] \"{}\" ({}x{})", l.index, l.name, l.width, l.height))
            .collect::<Vec<_>>()
            .join("\n")
    );

    output::print_output(format, &human, &info);
    0
}
