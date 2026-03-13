use crate::app::{OutputFormat, PipelineArgs};
use crate::output;
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::recipe;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct PipelineResult {
    input: String,
    output: String,
    steps: usize,
    output_size: u64,
}

pub fn run(args: &PipelineArgs, format: OutputFormat, dry_run: bool) -> i32 {
    let input = match &args.input {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: input".into(),
                suggestion:
                    "usage: panimg pipeline <input> -o <output> --steps \"grayscale | blur --sigma 2\""
                        .into(),
            };
            return output::print_error(format, &err);
        }
    };

    let output_path_str = match args.output.as_ref().or(args.output_pos.as_ref()) {
        Some(o) => o.clone(),
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: output (-o)".into(),
                suggestion:
                    "usage: panimg pipeline <input> -o <output> --steps \"grayscale | blur --sigma 2\""
                        .into(),
            };
            return output::print_error(format, &err);
        }
    };

    // Build pipeline from --steps or --recipe
    let pipeline = if let Some(steps) = &args.steps {
        match recipe::parse_steps(steps) {
            Ok(p) => p,
            Err(e) => return output::print_error(format, &e),
        }
    } else if let Some(recipe_path) = &args.recipe {
        let json_str = match std::fs::read_to_string(recipe_path) {
            Ok(s) => s,
            Err(e) => {
                let err = PanimgError::IoError {
                    message: e.to_string(),
                    path: Some(std::path::PathBuf::from(recipe_path)),
                    suggestion: "check that the recipe file exists and is readable".into(),
                };
                return output::print_error(format, &err);
            }
        };
        match recipe::parse_recipe(&json_str) {
            Ok(p) => p,
            Err(e) => return output::print_error(format, &e),
        }
    } else {
        let err = PanimgError::InvalidArgument {
            message: "pipeline requires --steps or --recipe".into(),
            suggestion: "use --steps \"grayscale | blur --sigma 2\" or --recipe recipe.json".into(),
        };
        return output::print_error(format, &err);
    };

    let input_path = Path::new(input);
    let output_path = Path::new(&output_path_str);

    if dry_run {
        let plan = pipeline.describe();
        output::print_output(
            format,
            &format!(
                "Would run {} steps on {} → {}",
                plan.steps.len(),
                input,
                output_path_str
            ),
            &plan,
        );
        return 0;
    }

    let img = match CodecRegistry::decode(input_path) {
        Ok(i) => i,
        Err(e) => return output::print_error(format, &e),
    };

    let step_count = pipeline.len();

    let result_img = match pipeline.execute(img) {
        Ok(i) => i,
        Err(e) => return output::print_error(format, &e),
    };

    let out_format = ImageFormat::from_path_extension(output_path)
        .or_else(|| ImageFormat::from_path(input_path))
        .unwrap_or(ImageFormat::Png);

    let options = EncodeOptions {
        format: out_format,
        quality: args.quality,
        strip_metadata: args.strip,
        resolution: None,
    };

    if let Err(e) = CodecRegistry::encode(&result_img, output_path, &options) {
        return output::print_error(format, &e);
    }

    let output_size = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);

    let result = PipelineResult {
        input: input.clone(),
        output: output_path_str,
        steps: step_count,
        output_size,
    };

    output::print_output(
        format,
        &format!(
            "Pipeline {} → {} ({} steps)",
            result.input, result.output, result.steps
        ),
        &result,
    );

    0
}
