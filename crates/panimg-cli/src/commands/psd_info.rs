use crate::app::{PsdInfoArgs, RunContext};
use panimg_core::error::PanimgError;
use std::path::Path;

pub fn run(args: &PsdInfoArgs, ctx: &RunContext) -> i32 {
    if ctx.schema {
        let schema = serde_json::json!({
            "command": "psd-info",
            "params": {
                "input": { "type": "string", "required": true, "description": "Input PSD file" }
            }
        });
        ctx.print_json(&schema);
        return 0;
    }

    let input = match &args.input {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: input".into(),
                suggestion: "usage: panimg psd-info <input.psd>".into(),
            };
            return ctx.print_error(&err);
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
            return ctx.print_error(&err);
        }
    };

    let info = match panimg_core::psd::get_psd_info(&data) {
        Ok(i) => i,
        Err(e) => return ctx.print_error(&e),
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

    ctx.print_output(&human, &info);
    0
}
