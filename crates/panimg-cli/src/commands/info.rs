use crate::app::{InfoArgs, OutputFormat};
use crate::output;
use panimg_core::error::PanimgError;
use panimg_core::info::ImageInfo;
use panimg_core::schema::{CommandSchema, ParamSchema, ParamType};
use std::path::Path;

pub fn schema() -> CommandSchema {
    CommandSchema {
        command: "info".into(),
        description: "Show image metadata and properties".into(),
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
                name: "fields".into(),
                param_type: ParamType::String,
                required: false,
                description: "Comma-separated list of fields to include in output".into(),
                default: None,
                choices: Some(vec![
                    "path".into(),
                    "format".into(),
                    "width".into(),
                    "height".into(),
                    "color_type".into(),
                    "bit_depth".into(),
                    "file_size".into(),
                    "has_alpha".into(),
                    "exif".into(),
                ]),
                range: None,
            },
        ],
    }
}

pub fn run(args: &InfoArgs, format: OutputFormat, show_schema: bool) -> i32 {
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
                suggestion: "usage: panimg info <file>".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let path = Path::new(input);
    let info = match ImageInfo::from_path(path) {
        Ok(i) => i,
        Err(e) => return output::print_error(format, &e),
    };

    let fields: Vec<String> = args
        .fields
        .as_ref()
        .map(|f| f.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    match format {
        OutputFormat::Human => {
            println!("{}", info.to_human_string());
        }
        OutputFormat::Json => {
            let json = info.to_filtered_json(&fields);
            output::print_json(&json);
        }
    }

    0
}
