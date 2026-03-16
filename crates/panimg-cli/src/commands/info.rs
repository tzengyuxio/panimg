use crate::app::{InfoArgs, OutputFormat, RunContext};
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
                choices: Some({
                    let mut fields = vec![
                        "path".into(),
                        "format".into(),
                        "width".into(),
                        "height".into(),
                        "color_type".into(),
                        "bit_depth".into(),
                        "file_size".into(),
                        "has_alpha".into(),
                        "page_count".into(),
                        "exif".into(),
                    ];
                    #[cfg(feature = "icc")]
                    fields.push("icc_profile".into());
                    fields
                }),
                range: None,
            },
        ],
    }
}

pub fn run(args: &InfoArgs, ctx: &RunContext) -> i32 {
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
                suggestion: "usage: panimg info <file>".into(),
            };
            return ctx.print_error(&err);
        }
    };

    let path = Path::new(input);
    let info = match ImageInfo::from_path(path) {
        Ok(i) => i,
        Err(e) => return ctx.print_error(&e),
    };

    let fields: Vec<String> = args
        .fields
        .as_ref()
        .map(|f| f.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    match ctx.format {
        OutputFormat::Human => {
            println!("{}", info.to_human_string(&fields));
        }
        OutputFormat::Json => {
            let json = info.to_filtered_json(&fields);
            ctx.print_json(&json);
        }
    }

    0
}
