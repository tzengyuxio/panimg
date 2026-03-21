use super::common::require_input;
use super::CommandResult;
use crate::app::{PdfPagesArgs, RunContext};
use panimg_core::codec::EncodeOptions;
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::pdf::{PageRange, PdfDocument};
use panimg_core::schema::{CommandSchema, ParamRange, ParamSchema, ParamType};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct PdfPagesResult {
    input: String,
    total_pages: usize,
    extracted: usize,
    output_dir: String,
    pages: Vec<PageOutput>,
}

#[derive(Serialize)]
struct PageOutput {
    page: usize, // 1-based for user display
    width: u32,
    height: u32,
    path: String,
}

pub fn schema() -> CommandSchema {
    CommandSchema {
        command: "pdf-pages".into(),
        description: "Extract individual pages from a PDF file".into(),
        params: vec![
            ParamSchema {
                name: "input".into(),
                param_type: ParamType::Path,
                required: true,
                description: "Input PDF file path".into(),
                default: None,
                choices: None,
                range: None,
            },
            ParamSchema {
                name: "output_dir".into(),
                param_type: ParamType::Path,
                required: false,
                description: "Output directory for extracted pages".into(),
                default: Some(serde_json::json!(".")),
                choices: None,
                range: None,
            },
            ParamSchema {
                name: "page_format".into(),
                param_type: ParamType::String,
                required: false,
                description: "Output format for pages (e.g. png, jpg)".into(),
                default: Some(serde_json::json!("png")),
                choices: None,
                range: None,
            },
            ParamSchema {
                name: "pages".into(),
                param_type: ParamType::String,
                required: false,
                description: "Page range to extract (e.g. '1-3,5')".into(),
                default: None,
                choices: None,
                range: None,
            },
            ParamSchema {
                name: "prefix".into(),
                param_type: ParamType::String,
                required: false,
                description: "Filename prefix for pages".into(),
                default: Some(serde_json::json!("page")),
                choices: None,
                range: None,
            },
            ParamSchema {
                name: "quality".into(),
                param_type: ParamType::Integer,
                required: false,
                description: "Output quality (1-100, for lossy formats)".into(),
                default: None,
                choices: None,
                range: Some(ParamRange {
                    min: 1.0,
                    max: 100.0,
                }),
            },
        ],
    }
}

pub fn run(args: &PdfPagesArgs, ctx: &RunContext) -> CommandResult {
    let input = require_input(
        &args.input,
        "panimg pdf-pages <input.pdf> --output-dir ./pages",
    )?;

    let input_path = Path::new(input);

    if ctx.dry_run {
        let plan = serde_json::json!({
            "operation": "pdf-pages",
            "input": input,
        });
        ctx.print_output(&format!("Would extract pages from {input}"), &plan);
        return Ok(0);
    }

    let data = std::fs::read(input_path).map_err(|e| PanimgError::IoError {
        message: e.to_string(),
        path: Some(input_path.to_path_buf()),
        suggestion: "check that the file exists and is readable".into(),
    })?;

    // Parse the PDF once — used for both page count and rendering
    let doc = PdfDocument::from_bytes(&data)?;

    let range = match &args.pages {
        Some(spec) => PageRange::parse(spec)?,
        None => PageRange::all(doc.page_count()),
    };

    let output_dir = args.output_dir.as_deref().unwrap_or(".");
    let out_dir = Path::new(output_dir);
    std::fs::create_dir_all(out_dir).map_err(|e| PanimgError::IoError {
        message: e.to_string(),
        path: Some(out_dir.to_path_buf()),
        suggestion: "check the output directory path".into(),
    })?;

    let ext = &args.page_format;
    let out_format =
        ImageFormat::from_extension(ext).ok_or_else(|| PanimgError::InvalidArgument {
            message: format!("unsupported page format: {ext}"),
            suggestion: "use png, jpg, webp, etc.".into(),
        })?;

    let prefix = args.prefix.as_deref().unwrap_or("page");
    let render_dpi = ctx.dpi.unwrap_or(panimg_core::codec::DEFAULT_DPI);

    let encode_opts = EncodeOptions {
        format: out_format,
        quality: args.quality,
        strip_metadata: false,
        resolution: None,
    };

    let mut page_outputs = Vec::new();
    let mut encode_error: Option<PanimgError> = None;

    let total_pages = doc.for_each_page(&range, render_dpi, |info, img| {
        let page_1based = info.index + 1;
        let filename = format!("{prefix}_{page_1based:04}.{ext}");
        let page_path = out_dir.join(&filename);

        if let Err(e) = panimg_core::codec::CodecRegistry::encode(&img, &page_path, &encode_opts) {
            encode_error = Some(e);
            return Ok(false);
        }

        page_outputs.push(PageOutput {
            page: page_1based,
            width: info.width,
            height: info.height,
            path: page_path.to_string_lossy().to_string(),
        });

        Ok(true)
    })?;

    if let Some(e) = encode_error {
        return Err(e);
    }

    let result = PdfPagesResult {
        input: input.to_string(),
        total_pages,
        extracted: page_outputs.len(),
        output_dir: output_dir.to_string(),
        pages: page_outputs,
    };

    ctx.print_output(
        &format!(
            "Extracted {}/{} pages from {} → {}",
            result.extracted, result.total_pages, result.input, result.output_dir
        ),
        &result,
    );

    Ok(0)
}
