use crate::app::RunContext;
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::pipeline::Pipeline;
use std::path::Path;

pub fn require_input<'a>(input: &'a Option<String>, usage: &str) -> Result<&'a str, PanimgError> {
    input
        .as_deref()
        .ok_or_else(|| PanimgError::InvalidArgument {
            message: "missing required argument: input".into(),
            suggestion: format!("usage: {usage}"),
        })
}

pub fn require_output(
    output: &Option<String>,
    output_pos: &Option<String>,
    usage: &str,
) -> Result<String, PanimgError> {
    output
        .as_ref()
        .or(output_pos.as_ref())
        .cloned()
        .ok_or_else(|| PanimgError::InvalidArgument {
            message: "missing required argument: output (-o)".into(),
            suggestion: format!("usage: {usage}"),
        })
}

pub struct PipelineInput<'a> {
    pub input_path: &'a Path,
    pub output_path: &'a Path,
    pub quality: Option<u8>,
    pub strip_metadata: bool,
}

pub struct PipelineOutput {
    pub original_width: u32,
    pub original_height: u32,
    pub new_width: u32,
    pub new_height: u32,
    pub output_size: u64,
}

/// Execute standard decode → pipeline → encode flow.
/// Returns `None` on dry-run (plan already printed), `Some(output)` on success.
pub fn run_pipeline(
    pipeline: &Pipeline,
    input: &PipelineInput,
    ctx: &RunContext,
) -> Result<Option<PipelineOutput>, PanimgError> {
    if ctx.dry_run {
        let plan = pipeline.describe();
        ctx.print_output(
            &format!(
                "Would process {} → {}",
                input.input_path.display(),
                input.output_path.display()
            ),
            &plan,
        );
        return Ok(None);
    }

    let img = CodecRegistry::decode_with_options(input.input_path, &ctx.decode_options())?;
    let (orig_w, orig_h) = (img.width(), img.height());

    let result_img = pipeline.execute(img)?;
    let (new_w, new_h) = (result_img.width(), result_img.height());

    let out_format = ImageFormat::from_path_extension(input.output_path)
        .or_else(|| ImageFormat::from_path(input.input_path))
        .unwrap_or(ImageFormat::Png);

    let options = EncodeOptions {
        format: out_format,
        quality: input.quality,
        strip_metadata: input.strip_metadata,
        resolution: None,
    };
    CodecRegistry::encode(&result_img, input.output_path, &options)?;

    let output_size = std::fs::metadata(input.output_path)
        .map(|m| m.len())
        .unwrap_or(0);

    Ok(Some(PipelineOutput {
        original_width: orig_w,
        original_height: orig_h,
        new_width: new_w,
        new_height: new_h,
        output_size,
    }))
}
