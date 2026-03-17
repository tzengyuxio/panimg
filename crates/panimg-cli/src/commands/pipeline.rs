use super::common::{require_input, require_output};
use super::CommandResult;
use crate::app::{PipelineArgs, RunContext};
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

pub fn run(args: &PipelineArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(
        &args.input,
        "panimg pipeline <input> -o <output> --steps \"grayscale | blur --sigma 2\"",
    )?;
    let output_path_str = require_output(
        &args.output,
        &args.output_pos,
        "panimg pipeline <input> -o <output> --steps \"grayscale | blur --sigma 2\"",
    )?;

    // Build pipeline from --steps or --recipe
    let pipeline = if let Some(steps) = &args.steps {
        recipe::parse_steps(steps)?
    } else if let Some(recipe_path) = &args.recipe {
        let json_str = std::fs::read_to_string(recipe_path).map_err(|e| PanimgError::IoError {
            message: e.to_string(),
            path: Some(std::path::PathBuf::from(recipe_path)),
            suggestion: "check that the recipe file exists and is readable".into(),
        })?;
        recipe::parse_recipe(&json_str)?
    } else {
        return Err(PanimgError::InvalidArgument {
            message: "pipeline requires --steps or --recipe".into(),
            suggestion: "use --steps \"grayscale | blur --sigma 2\" or --recipe recipe.json".into(),
        });
    };

    let input_path = Path::new(input);
    let output_path = Path::new(&output_path_str);

    if ctx.dry_run {
        let plan = pipeline.describe();
        ctx.print_output(
            &format!(
                "Would run {} steps on {} → {}",
                plan.steps.len(),
                input,
                output_path_str
            ),
            &plan,
        );
        return Ok(0);
    }

    let img = CodecRegistry::decode_with_options(input_path, &ctx.decode_options())?;

    let step_count = pipeline.len();
    let result_img = pipeline.execute(img)?;

    let out_format = ImageFormat::from_path_extension(output_path)
        .or_else(|| ImageFormat::from_path(input_path))
        .unwrap_or(ImageFormat::Png);

    let options = EncodeOptions {
        format: out_format,
        quality: args.quality,
        strip_metadata: args.strip,
        resolution: None,
    };

    CodecRegistry::encode(&result_img, output_path, &options)?;

    let output_size = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);

    let result = PipelineResult {
        input: input.to_string(),
        output: output_path_str,
        steps: step_count,
        output_size,
    };

    ctx.print_output(
        &format!(
            "Pipeline {} → {} ({} steps)",
            result.input, result.output, result.steps
        ),
        &result,
    );

    Ok(0)
}
