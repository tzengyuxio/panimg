use crate::app::{OutputFormat, TextArgs};
use crate::output;
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::ops::draw::parse_color;
use panimg_core::ops::text::DrawTextOp;
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct TextResult {
    input: String,
    output: String,
    content: String,
    size: f32,
    output_size: u64,
}

pub fn run(args: &TextArgs, format: OutputFormat, dry_run: bool, show_schema: bool) -> i32 {
    if show_schema {
        let s = DrawTextOp::schema();
        output::print_json(&serde_json::to_value(&s).unwrap());
        return 0;
    }

    let input = match &args.input {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: input".into(),
                suggestion: "usage: panimg text <input> -o <output> --content \"Hello\"".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let content = match &args.content {
        Some(c) => c.clone(),
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: --content".into(),
                suggestion: "usage: panimg text <input> -o <output> --content \"Hello\"".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let output_path_str = match args.output.as_ref().or(args.output_pos.as_ref()) {
        Some(o) => o.clone(),
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: output (-o)".into(),
                suggestion: "usage: panimg text <input> -o <output> --content \"Hello\"".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let size = args.size.unwrap_or(24.0);
    let color_str = args.color.as_deref().unwrap_or("white");
    let color = match parse_color(color_str) {
        Ok(c) => c,
        Err(e) => return output::print_error(format, &e),
    };
    let margin = args.margin.unwrap_or(10);

    let text_op = match DrawTextOp::new(
        content.clone(),
        args.font.as_deref(),
        size,
        color,
        args.x,
        args.y,
        args.position.clone(),
        margin,
    ) {
        Ok(op) => op,
        Err(e) => return output::print_error(format, &e),
    };

    if dry_run {
        let desc = text_op.describe();
        output::print_output(
            format,
            &format!(
                "Would draw text \"{}\" on {} → {} (size={})",
                content, input, output_path_str, size
            ),
            &desc,
        );
        return 0;
    }

    let input_path = Path::new(input);
    let output_path = Path::new(&output_path_str);

    let img = match CodecRegistry::decode(input_path) {
        Ok(i) => i,
        Err(e) => return output::print_error(format, &e),
    };

    let pipeline = Pipeline::new().push(text_op);

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
    };

    if let Err(e) = CodecRegistry::encode(&result_img, output_path, &options) {
        return output::print_error(format, &e);
    }

    let output_size = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);

    let result = TextResult {
        input: input.clone(),
        output: output_path_str,
        content,
        size,
        output_size,
    };

    output::print_output(
        format,
        &format!(
            "Text \"{}\" → {} (size={}, {})",
            result.content,
            result.output,
            result.size,
            humanize_bytes(result.output_size)
        ),
        &result,
    );

    0
}

fn humanize_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
