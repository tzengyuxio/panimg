use super::common::require_input;
use super::CommandResult;
use crate::app::{PsdInfoArgs, RunContext};
use panimg_core::error::PanimgError;
use panimg_core::schema::{CommandSchema, ParamSchema, ParamType};
use std::path::Path;

pub fn schema() -> CommandSchema {
    CommandSchema {
        command: "psd-info".into(),
        description: "Show PSD layer metadata".into(),
        params: vec![ParamSchema {
            name: "input".into(),
            param_type: ParamType::Path,
            required: true,
            description: "Input PSD file".into(),
            default: None,
            choices: None,
            range: None,
        }],
    }
}

pub fn run(args: &PsdInfoArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(&args.input, "panimg psd-info <input.psd>")?;

    let path = Path::new(input);
    let data = std::fs::read(path).map_err(|e| PanimgError::IoError {
        message: e.to_string(),
        path: Some(path.to_path_buf()),
        suggestion: "check that the file exists and is readable".into(),
    })?;

    let info = panimg_core::psd::get_psd_info(&data)?;

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
    Ok(0)
}
