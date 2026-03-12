use crate::app::{DrawArgs, OutputFormat};
use crate::output;
use panimg_core::codec::{CodecRegistry, EncodeOptions};
use panimg_core::color::parse_color;
use panimg_core::error::PanimgError;
use panimg_core::format::ImageFormat;
use panimg_core::ops::draw::{DrawCircleOp, DrawLineOp, DrawRectOp};
use panimg_core::ops::Operation;
use panimg_core::pipeline::Pipeline;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct DrawResult {
    input: String,
    output: String,
    shape: String,
    output_size: u64,
}

pub fn run(args: &DrawArgs, format: OutputFormat, dry_run: bool, show_schema: bool) -> i32 {
    if show_schema {
        let s = DrawRectOp::schema();
        output::print_json(&serde_json::to_value(&s).unwrap());
        return 0;
    }

    let input = match &args.input {
        Some(i) => i,
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: input".into(),
                suggestion: "usage: panimg draw <input> -o <output> --shape rect --x 10 --y 10 --width 100 --height 50".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let output_path_str = match args.output.as_ref().or(args.output_pos.as_ref()) {
        Some(o) => o.clone(),
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: output (-o)".into(),
                suggestion: "usage: panimg draw <input> -o <output> --shape rect ...".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let shape = match &args.shape {
        Some(s) => s.as_str(),
        None => {
            let err = PanimgError::InvalidArgument {
                message: "missing required argument: --shape".into(),
                suggestion: "use --shape rect, --shape circle, or --shape line".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let color_str = args.color.as_deref().unwrap_or("red");
    let color = match parse_color(color_str) {
        Ok(c) => c,
        Err(e) => return output::print_error(format, &e),
    };

    let fill = args.fill;
    let thickness = args.thickness.unwrap_or(2);

    let pipeline = match shape {
        "rect" => {
            let x = args.x.unwrap_or(0);
            let y = args.y.unwrap_or(0);
            let width = match args.width {
                Some(w) => w,
                None => {
                    let err = PanimgError::InvalidArgument {
                        message: "rect requires --width".into(),
                        suggestion: "e.g. --shape rect --x 10 --y 10 --width 100 --height 50"
                            .into(),
                    };
                    return output::print_error(format, &err);
                }
            };
            let height = match args.height {
                Some(h) => h,
                None => {
                    let err = PanimgError::InvalidArgument {
                        message: "rect requires --height".into(),
                        suggestion: "e.g. --shape rect --x 10 --y 10 --width 100 --height 50"
                            .into(),
                    };
                    return output::print_error(format, &err);
                }
            };
            let op = match DrawRectOp::new(x, y, width, height, color, fill, thickness) {
                Ok(op) => op,
                Err(e) => return output::print_error(format, &e),
            };
            Pipeline::new().push(op)
        }
        "circle" => {
            let cx = args.cx.or(args.x).unwrap_or(0);
            let cy = args.cy.or(args.y).unwrap_or(0);
            let radius = match args.radius {
                Some(r) => r,
                None => {
                    let err = PanimgError::InvalidArgument {
                        message: "circle requires --radius".into(),
                        suggestion: "e.g. --shape circle --cx 50 --cy 50 --radius 30".into(),
                    };
                    return output::print_error(format, &err);
                }
            };
            let op = match DrawCircleOp::new(cx, cy, radius, color, fill, thickness) {
                Ok(op) => op,
                Err(e) => return output::print_error(format, &e),
            };
            Pipeline::new().push(op)
        }
        "line" => {
            let x1 = args.x1.or(args.x).unwrap_or(0);
            let y1 = args.y1.or(args.y).unwrap_or(0);
            let x2 = match args.x2 {
                Some(v) => v,
                None => {
                    let err = PanimgError::InvalidArgument {
                        message: "line requires --x2".into(),
                        suggestion: "e.g. --shape line --x1 0 --y1 0 --x2 100 --y2 100".into(),
                    };
                    return output::print_error(format, &err);
                }
            };
            let y2 = match args.y2 {
                Some(v) => v,
                None => {
                    let err = PanimgError::InvalidArgument {
                        message: "line requires --y2".into(),
                        suggestion: "e.g. --shape line --x1 0 --y1 0 --x2 100 --y2 100".into(),
                    };
                    return output::print_error(format, &err);
                }
            };
            Pipeline::new().push(DrawLineOp::new(x1, y1, x2, y2, color, thickness))
        }
        _ => {
            let err = PanimgError::InvalidArgument {
                message: format!("unknown shape: '{shape}'"),
                suggestion: "use --shape rect, --shape circle, or --shape line".into(),
            };
            return output::print_error(format, &err);
        }
    };

    let input_path = Path::new(input);
    let output_path = Path::new(&output_path_str);

    if dry_run {
        let plan = pipeline.describe();
        output::print_output(
            format,
            &format!("Would draw {shape} on {} → {}", input, output_path_str),
            &plan,
        );
        return 0;
    }

    let img = match CodecRegistry::decode(input_path) {
        Ok(i) => i,
        Err(e) => return output::print_error(format, &e),
    };

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

    let result = DrawResult {
        input: input.clone(),
        output: output_path_str,
        shape: shape.to_string(),
        output_size,
    };

    output::print_output(
        format,
        &format!("Drew {shape} on {} → {}", result.input, result.output),
        &result,
    );

    0
}
