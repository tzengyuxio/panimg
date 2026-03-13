use crate::color::parse_color;
use crate::error::{PanimgError, Result};
use crate::ops::blur::BlurOp;
use crate::ops::brightness::BrightnessOp;
use crate::ops::color::{PosterizeOp, SaturateOp, SepiaOp};
use crate::ops::contrast::ContrastOp;
use crate::ops::crop::CropOp;
use crate::ops::edge_detect::EdgeDetectOp;
use crate::ops::emboss::EmbossOp;
use crate::ops::flip::{FlipDirection, FlipOp};
use crate::ops::grayscale::GrayscaleOp;
use crate::ops::hue_rotate::HueRotateOp;
use crate::ops::invert::InvertOp;
#[cfg(feature = "text")]
use crate::ops::position::Position;
use crate::ops::resize::{FitMode, ResizeFilter, ResizeOp};
use crate::ops::rotate::{RotateAngle, RotateOp};
use crate::ops::sharpen::SharpenOp;
use crate::ops::smart_crop::{SmartCropOp, SmartCropStrategy};
#[cfg(feature = "text")]
use crate::ops::text::DrawTextOp;
use crate::ops::tilt_shift::TiltShiftOp;
use crate::ops::trim::TrimOp;
use crate::pipeline::Pipeline;
use image::DynamicImage;
use pan_common::pipeline::Operation;
use serde::Deserialize;
use serde_json::{Map, Value};

/// A single step in a recipe, parsed from JSON.
#[derive(Debug, Deserialize)]
pub struct RecipeStep {
    pub op: String,
    #[serde(flatten)]
    pub params: Map<String, Value>,
}

/// A recipe: a list of steps to apply in order.
#[derive(Debug, Deserialize)]
pub struct Recipe {
    pub steps: Vec<RecipeStep>,
}

/// Parse a pipe-separated steps string into a Pipeline.
///
/// Format: "resize --width 800 | blur --sigma 2 | grayscale"
pub fn parse_steps(steps_str: &str) -> Result<Pipeline> {
    let mut pipeline = Pipeline::new();

    for step in steps_str.split('|') {
        let step = step.trim();
        if step.is_empty() {
            continue;
        }
        let op = parse_single_step(step)?;
        pipeline = pipeline.push_boxed(op);
    }

    if pipeline.is_empty() {
        return Err(PanimgError::InvalidArgument {
            message: "pipeline has no steps".into(),
            suggestion: "provide at least one step, e.g. --steps \"grayscale | blur --sigma 2\""
                .into(),
        });
    }

    Ok(pipeline)
}

/// Parse a recipe from JSON content.
pub fn parse_recipe(json_str: &str) -> Result<Pipeline> {
    let recipe: Recipe =
        serde_json::from_str(json_str).map_err(|e| PanimgError::InvalidArgument {
            message: format!("invalid recipe JSON: {e}"),
            suggestion: "recipe must be JSON with a \"steps\" array".into(),
        })?;

    let mut pipeline = Pipeline::new();
    for step in recipe.steps {
        let op = build_op_from_recipe_step(&step)?;
        pipeline = pipeline.push_boxed(op);
    }

    if pipeline.is_empty() {
        return Err(PanimgError::InvalidArgument {
            message: "recipe has no steps".into(),
            suggestion: "provide at least one step in the \"steps\" array".into(),
        });
    }

    Ok(pipeline)
}

/// Parse a single inline step like "resize --width 800 --height 600".
fn parse_single_step(step: &str) -> Result<Box<dyn Operation<DynamicImage, PanimgError>>> {
    let parts: Vec<&str> = step.split_whitespace().collect();
    if parts.is_empty() {
        return Err(PanimgError::InvalidArgument {
            message: "empty step".into(),
            suggestion: "each step needs at least an operation name".into(),
        });
    }

    let op_name = parts[0];
    let args = &parts[1..];

    match op_name {
        "grayscale" => Ok(Box::new(GrayscaleOp::new())),
        "invert" => Ok(Box::new(InvertOp::new())),
        "edge-detect" => Ok(Box::new(EdgeDetectOp::new())),
        "emboss" => Ok(Box::new(EmbossOp::new())),
        "blur" => {
            let sigma = parse_f32_arg(args, "--sigma")?
                .ok_or_else(|| PanimgError::InvalidArgument {
                    message: "blur requires --sigma".into(),
                    suggestion: "e.g. blur --sigma 2.0".into(),
                })?;
            Ok(Box::new(BlurOp::new(sigma)?))
        }
        "sharpen" => {
            let sigma = parse_f32_arg(args, "--sigma")?
                .ok_or_else(|| PanimgError::InvalidArgument {
                    message: "sharpen requires --sigma".into(),
                    suggestion: "e.g. sharpen --sigma 1.5".into(),
                })?;
            let threshold = parse_i32_arg(args, "--threshold")?.unwrap_or(0);
            Ok(Box::new(SharpenOp::new(sigma, threshold)?))
        }
        "brightness" => {
            let value = parse_i32_arg(args, "--value")?
                .ok_or_else(|| PanimgError::InvalidArgument {
                    message: "brightness requires --value".into(),
                    suggestion: "e.g. brightness --value 20".into(),
                })?;
            Ok(Box::new(BrightnessOp::new(value)?))
        }
        "contrast" => {
            let value = parse_f32_arg(args, "--value")?
                .ok_or_else(|| PanimgError::InvalidArgument {
                    message: "contrast requires --value".into(),
                    suggestion: "e.g. contrast --value 1.5".into(),
                })?;
            Ok(Box::new(ContrastOp::new(value)?))
        }
        "hue-rotate" => {
            let degrees = parse_i32_arg(args, "--degrees")?
                .ok_or_else(|| PanimgError::InvalidArgument {
                    message: "hue-rotate requires --degrees".into(),
                    suggestion: "e.g. hue-rotate --degrees 90".into(),
                })?;
            Ok(Box::new(HueRotateOp::new(degrees)?))
        }
        "resize" => {
            let width = parse_u32_arg(args, "--width")?;
            let height = parse_u32_arg(args, "--height")?;
            let fit = parse_str_arg(args, "--fit")
                .map(|s| FitMode::parse(&s))
                .transpose()?
                .unwrap_or(FitMode::Contain);
            let filter = parse_str_arg(args, "--filter")
                .map(|s| ResizeFilter::parse(&s))
                .transpose()?
                .unwrap_or(ResizeFilter::Lanczos3);
            Ok(Box::new(ResizeOp::new(width, height, fit, filter)?))
        }
        "crop" => {
            let x = parse_u32_arg(args, "--x")?.unwrap_or(0);
            let y = parse_u32_arg(args, "--y")?.unwrap_or(0);
            let width = parse_u32_arg(args, "--width")?
                .ok_or_else(|| PanimgError::InvalidArgument {
                    message: "crop requires --width".into(),
                    suggestion: "e.g. crop --x 10 --y 10 --width 100 --height 100".into(),
                })?;
            let height = parse_u32_arg(args, "--height")?
                .ok_or_else(|| PanimgError::InvalidArgument {
                    message: "crop requires --height".into(),
                    suggestion: "e.g. crop --x 10 --y 10 --width 100 --height 100".into(),
                })?;
            Ok(Box::new(CropOp::new(x, y, width, height)?))
        }
        "rotate" => {
            let angle_str = parse_str_arg(args, "--angle")
                .ok_or_else(|| PanimgError::InvalidArgument {
                    message: "rotate requires --angle".into(),
                    suggestion: "e.g. rotate --angle 90".into(),
                })?;
            let mut op = RotateOp::new(RotateAngle::parse(&angle_str)?);
            if let Some(bg_str) = parse_str_arg(args, "--background") {
                op = op.with_background(parse_color(&bg_str)?);
            }
            Ok(Box::new(op))
        }
        "flip" => {
            let direction = parse_str_arg(args, "--direction")
                .ok_or_else(|| PanimgError::InvalidArgument {
                    message: "flip requires --direction".into(),
                    suggestion: "e.g. flip --direction horizontal".into(),
                })?;
            Ok(Box::new(FlipOp::new(FlipDirection::parse(&direction)?)))
        }
        "trim" => {
            let tolerance = parse_u32_arg(args, "--tolerance")?
                .map(|v| v as u8)
                .unwrap_or(10);
            Ok(Box::new(TrimOp::new(tolerance)?))
        }
        "saturate" => {
            let factor = parse_f32_arg(args, "--factor")?
                .ok_or_else(|| PanimgError::InvalidArgument {
                    message: "saturate requires --factor".into(),
                    suggestion: "e.g. saturate --factor 1.5".into(),
                })?;
            Ok(Box::new(SaturateOp::new(factor)?))
        }
        "sepia" => {
            let intensity = parse_f32_arg(args, "--intensity")?.unwrap_or(1.0);
            Ok(Box::new(SepiaOp::new(intensity)?))
        }
        "posterize" => {
            let levels = parse_u32_arg(args, "--levels")?
                .map(|v| v as u8)
                .unwrap_or(4);
            Ok(Box::new(PosterizeOp::new(levels)?))
        }
        "tilt-shift" => {
            let sigma = parse_f32_arg(args, "--sigma")?.unwrap_or(8.0);
            let focus_position = parse_f32_arg(args, "--focus-position")?.unwrap_or(0.5);
            let focus_width = parse_f32_arg(args, "--focus-width")?.unwrap_or(0.15);
            let transition = parse_f32_arg(args, "--transition")?.unwrap_or(0.2);
            let saturation = parse_f32_arg(args, "--saturation")?.unwrap_or(1.0);
            Ok(Box::new(TiltShiftOp::new(
                sigma,
                focus_position,
                focus_width,
                transition,
                saturation,
            )?))
        }
        "smart-crop" => {
            let width = parse_u32_arg(args, "--width")?
                .ok_or_else(|| PanimgError::InvalidArgument {
                    message: "smart-crop requires --width".into(),
                    suggestion: "e.g. smart-crop --width 200 --height 200".into(),
                })?;
            let height = parse_u32_arg(args, "--height")?
                .ok_or_else(|| PanimgError::InvalidArgument {
                    message: "smart-crop requires --height".into(),
                    suggestion: "e.g. smart-crop --width 200 --height 200".into(),
                })?;
            let strategy = parse_str_arg(args, "--strategy")
                .map(|s| SmartCropStrategy::parse(&s))
                .transpose()?
                .unwrap_or(SmartCropStrategy::Entropy);
            let step = parse_u32_arg(args, "--step")?;
            Ok(Box::new(SmartCropOp::new(width, height, strategy, step)?))
        }
        #[cfg(feature = "text")]
        "text" => {
            let content = parse_str_arg(args, "--content")
                .ok_or_else(|| PanimgError::InvalidArgument {
                    message: "text requires --content".into(),
                    suggestion: "e.g. text --content \"Hello World\"".into(),
                })?;
            let font = parse_str_arg(args, "--font");
            let size = parse_f32_arg(args, "--size")?.unwrap_or(24.0);
            let color_str = parse_str_arg(args, "--color").unwrap_or_else(|| "white".into());
            let color = parse_color(&color_str)?;
            let x = parse_i32_arg(args, "--x")?;
            let y = parse_i32_arg(args, "--y")?;
            let position = parse_str_arg(args, "--position")
                .map(|s| s.parse::<Position>())
                .transpose()?;
            let margin = parse_u32_arg(args, "--margin")?.unwrap_or(10);
            Ok(Box::new(DrawTextOp::new(
                content,
                font.as_deref(),
                size,
                color,
                x,
                y,
                position,
                margin,
            )?))
        }
        _ => Err(PanimgError::InvalidArgument {
            message: format!("unknown pipeline operation: '{op_name}'"),
            suggestion: "supported: grayscale, invert, blur, sharpen, brightness, contrast, hue-rotate, resize, crop, rotate, flip, edge-detect, emboss, trim, saturate, sepia, posterize, tilt-shift, smart-crop, text".into(),
        }),
    }
}

// --- Helpers for extracting typed values from a JSON params Map ---

fn get_f32(params: &Map<String, Value>, key: &str) -> Result<Option<f32>> {
    match params.get(key) {
        None => Ok(None),
        Some(v) => v
            .as_f64()
            .map(|f| Some(f as f32))
            .ok_or_else(|| PanimgError::InvalidArgument {
                message: format!("'{key}' must be a number"),
                suggestion: format!("e.g. \"{key}\": 2.0"),
            }),
    }
}

fn require_f32(params: &Map<String, Value>, key: &str, op: &str) -> Result<f32> {
    get_f32(params, key)?.ok_or_else(|| PanimgError::InvalidArgument {
        message: format!("{op} requires \"{key}\""),
        suggestion: format!("{{\"op\": \"{op}\", \"{key}\": 2.0}}"),
    })
}

fn get_u32(params: &Map<String, Value>, key: &str) -> Result<Option<u32>> {
    match params.get(key) {
        None => Ok(None),
        Some(v) => v
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .map(Some)
            .ok_or_else(|| PanimgError::InvalidArgument {
                message: format!("'{key}' must be a positive integer"),
                suggestion: format!("e.g. \"{key}\": 100"),
            }),
    }
}

fn require_u32(params: &Map<String, Value>, key: &str, op: &str) -> Result<u32> {
    get_u32(params, key)?.ok_or_else(|| PanimgError::InvalidArgument {
        message: format!("{op} requires \"{key}\""),
        suggestion: format!("{{\"op\": \"{op}\", \"{key}\": 100}}"),
    })
}

fn get_i32(params: &Map<String, Value>, key: &str) -> Result<Option<i32>> {
    match params.get(key) {
        None => Ok(None),
        Some(v) => v
            .as_i64()
            .and_then(|n| i32::try_from(n).ok())
            .map(Some)
            .ok_or_else(|| PanimgError::InvalidArgument {
                message: format!("'{key}' must be an integer"),
                suggestion: format!("e.g. \"{key}\": 10"),
            }),
    }
}

fn require_i32(params: &Map<String, Value>, key: &str, op: &str) -> Result<i32> {
    get_i32(params, key)?.ok_or_else(|| PanimgError::InvalidArgument {
        message: format!("{op} requires \"{key}\""),
        suggestion: format!("{{\"op\": \"{op}\", \"{key}\": 10}}"),
    })
}

fn get_u8(params: &Map<String, Value>, key: &str) -> Result<Option<u8>> {
    match params.get(key) {
        None => Ok(None),
        Some(v) => v
            .as_u64()
            .and_then(|n| u8::try_from(n).ok())
            .map(Some)
            .ok_or_else(|| PanimgError::InvalidArgument {
                message: format!("'{key}' must be an integer between 0 and 255"),
                suggestion: format!("e.g. \"{key}\": 10"),
            }),
    }
}

fn get_str<'a>(params: &'a Map<String, Value>, key: &str) -> Result<Option<&'a str>> {
    match params.get(key) {
        None => Ok(None),
        Some(v) => v
            .as_str()
            .map(Some)
            .ok_or_else(|| PanimgError::InvalidArgument {
                message: format!("'{key}' must be a string"),
                suggestion: format!("e.g. \"{key}\": \"value\""),
            }),
    }
}

fn require_str<'a>(params: &'a Map<String, Value>, key: &str, op: &str) -> Result<&'a str> {
    get_str(params, key)?.ok_or_else(|| PanimgError::InvalidArgument {
        message: format!("{op} requires \"{key}\""),
        suggestion: format!("{{\"op\": \"{op}\", \"{key}\": \"value\"}}"),
    })
}

/// Build an operation from a JSON recipe step.
fn build_op_from_recipe_step(
    step: &RecipeStep,
) -> Result<Box<dyn Operation<DynamicImage, PanimgError>>> {
    let p = &step.params;
    let op = step.op.as_str();

    match op {
        "grayscale" => Ok(Box::new(GrayscaleOp::new())),
        "invert" => Ok(Box::new(InvertOp::new())),
        "edge-detect" => Ok(Box::new(EdgeDetectOp::new())),
        "emboss" => Ok(Box::new(EmbossOp::new())),
        "blur" => {
            let sigma = require_f32(p, "sigma", op)?;
            Ok(Box::new(BlurOp::new(sigma)?))
        }
        "sharpen" => {
            let sigma = require_f32(p, "sigma", op)?;
            let threshold = get_i32(p, "threshold")?.unwrap_or(0);
            Ok(Box::new(SharpenOp::new(sigma, threshold)?))
        }
        "brightness" => {
            let value = require_i32(p, "value", op)?;
            Ok(Box::new(BrightnessOp::new(value)?))
        }
        "contrast" => {
            let value = require_f32(p, "contrast_value", op)?;
            Ok(Box::new(ContrastOp::new(value)?))
        }
        "hue-rotate" => {
            let degrees = require_i32(p, "degrees", op)?;
            Ok(Box::new(HueRotateOp::new(degrees)?))
        }
        "resize" => {
            let width = get_u32(p, "width")?;
            let height = get_u32(p, "height")?;
            let fit = get_str(p, "fit")?
                .map(FitMode::parse)
                .transpose()?
                .unwrap_or(FitMode::Contain);
            let filter = get_str(p, "filter")?
                .map(ResizeFilter::parse)
                .transpose()?
                .unwrap_or(ResizeFilter::Lanczos3);
            Ok(Box::new(ResizeOp::new(width, height, fit, filter)?))
        }
        "crop" => {
            let x = get_u32(p, "x")?.unwrap_or(0);
            let y = get_u32(p, "y")?.unwrap_or(0);
            let width = require_u32(p, "width", op)?;
            let height = require_u32(p, "height", op)?;
            Ok(Box::new(CropOp::new(x, y, width, height)?))
        }
        "rotate" => {
            let angle_str = require_str(p, "angle", op)?;
            let mut rotate_op = RotateOp::new(RotateAngle::parse(angle_str)?);
            if let Some(bg_str) = get_str(p, "background")? {
                rotate_op = rotate_op.with_background(parse_color(bg_str)?);
            }
            Ok(Box::new(rotate_op))
        }
        "flip" => {
            let direction = require_str(p, "direction", op)?;
            Ok(Box::new(FlipOp::new(FlipDirection::parse(direction)?)))
        }
        "trim" => {
            let tolerance = get_u8(p, "tolerance")?.unwrap_or(10);
            Ok(Box::new(TrimOp::new(tolerance)?))
        }
        "saturate" => {
            let factor = require_f32(p, "factor", op)?;
            Ok(Box::new(SaturateOp::new(factor)?))
        }
        "sepia" => {
            let intensity = get_f32(p, "intensity")?.unwrap_or(1.0);
            Ok(Box::new(SepiaOp::new(intensity)?))
        }
        "posterize" => {
            let levels = get_u8(p, "levels")?.unwrap_or(4);
            Ok(Box::new(PosterizeOp::new(levels)?))
        }
        "tilt-shift" => {
            let sigma = get_f32(p, "sigma")?.unwrap_or(8.0);
            let focus_position = get_f32(p, "focus_position")?.unwrap_or(0.5);
            let focus_width = get_f32(p, "focus_width")?.unwrap_or(0.15);
            let transition = get_f32(p, "transition")?.unwrap_or(0.2);
            let saturation = get_f32(p, "saturation")?.unwrap_or(1.0);
            Ok(Box::new(TiltShiftOp::new(
                sigma,
                focus_position,
                focus_width,
                transition,
                saturation,
            )?))
        }
        "smart-crop" => {
            let width = require_u32(p, "width", op)?;
            let height = require_u32(p, "height", op)?;
            let strategy = get_str(p, "strategy")?
                .map(SmartCropStrategy::parse)
                .transpose()?
                .unwrap_or(SmartCropStrategy::Entropy);
            let step = get_u32(p, "step")?;
            Ok(Box::new(SmartCropOp::new(width, height, strategy, step)?))
        }
        #[cfg(feature = "text")]
        "text" => {
            let content = require_str(p, "content", op)?.to_owned();
            let font = get_str(p, "font")?;
            let size = get_f32(p, "size")?.unwrap_or(24.0);
            let color_str = get_str(p, "color")?.unwrap_or("white");
            let color = parse_color(color_str)?;
            let x = get_i32(p, "x")?;
            let y = get_i32(p, "y")?;
            let position = get_str(p, "position")?
                .map(|s| s.parse::<Position>())
                .transpose()?;
            let margin = get_u32(p, "margin")?.unwrap_or(10);
            Ok(Box::new(DrawTextOp::new(
                content,
                font,
                size,
                color,
                x,
                y,
                position,
                margin,
            )?))
        }
        _ => Err(PanimgError::InvalidArgument {
            message: format!("unknown recipe operation: '{op}'"),
            suggestion: "supported: grayscale, invert, blur, sharpen, brightness, contrast, hue-rotate, resize, crop, rotate, flip, edge-detect, emboss, trim, saturate, sepia, posterize, tilt-shift, smart-crop, text".into(),
        }),
    }
}

// --- Argument parsing helpers for inline step syntax ---

fn parse_str_arg(args: &[&str], flag: &str) -> Option<String> {
    args.iter()
        .position(|&a| a == flag)
        .and_then(|i| args.get(i + 1))
        .map(|s| s.to_string())
}

fn parse_f32_arg(args: &[&str], flag: &str) -> Result<Option<f32>> {
    match parse_str_arg(args, flag) {
        Some(s) => s
            .parse::<f32>()
            .map(Some)
            .map_err(|_| PanimgError::InvalidArgument {
                message: format!("invalid float value for {flag}: '{s}'"),
                suggestion: format!("use a number like {flag} 2.0"),
            }),
        None => Ok(None),
    }
}

fn parse_i32_arg(args: &[&str], flag: &str) -> Result<Option<i32>> {
    match parse_str_arg(args, flag) {
        Some(s) => s
            .parse::<i32>()
            .map(Some)
            .map_err(|_| PanimgError::InvalidArgument {
                message: format!("invalid integer value for {flag}: '{s}'"),
                suggestion: format!("use an integer like {flag} 10"),
            }),
        None => Ok(None),
    }
}

fn parse_u32_arg(args: &[&str], flag: &str) -> Result<Option<u32>> {
    match parse_str_arg(args, flag) {
        Some(s) => s
            .parse::<u32>()
            .map(Some)
            .map_err(|_| PanimgError::InvalidArgument {
                message: format!("invalid positive integer value for {flag}: '{s}'"),
                suggestion: format!("use a positive integer like {flag} 100"),
            }),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_grayscale() {
        let pipeline = parse_steps("grayscale").unwrap();
        assert!(!pipeline.is_empty());
    }

    #[test]
    fn parse_multi_steps() {
        let pipeline = parse_steps("grayscale | blur --sigma 2.0 | invert").unwrap();
        let plan = pipeline.describe();
        assert_eq!(plan.steps.len(), 3);
        assert_eq!(plan.steps[0].operation, "grayscale");
        assert_eq!(plan.steps[1].operation, "blur");
        assert_eq!(plan.steps[2].operation, "invert");
    }

    #[test]
    fn parse_resize_step() {
        let pipeline = parse_steps("resize --width 100 --height 200").unwrap();
        let plan = pipeline.describe();
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].operation, "resize");
    }

    #[test]
    fn parse_empty_steps_error() {
        assert!(parse_steps("").is_err());
        assert!(parse_steps("  ").is_err());
    }

    #[test]
    fn parse_unknown_op_error() {
        assert!(parse_steps("nonexistent").is_err());
    }

    #[test]
    fn parse_recipe_json() {
        let json = r#"{"steps": [{"op": "grayscale"}, {"op": "blur", "sigma": 2.0}]}"#;
        let pipeline = parse_recipe(json).unwrap();
        let plan = pipeline.describe();
        assert_eq!(plan.steps.len(), 2);
    }

    #[test]
    fn parse_recipe_resize() {
        let json = r#"{"steps": [{"op": "resize", "width": 100, "fit": "cover"}]}"#;
        let pipeline = parse_recipe(json).unwrap();
        let plan = pipeline.describe();
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].operation, "resize");
    }

    #[test]
    fn parse_recipe_invalid_json() {
        assert!(parse_recipe("not json").is_err());
    }

    #[test]
    fn parse_recipe_empty_steps() {
        assert!(parse_recipe(r#"{"steps": []}"#).is_err());
    }

    #[test]
    fn parse_complex_pipeline() {
        let steps = "resize --width 800 | blur --sigma 1.5 | brightness --value 10 | sharpen --sigma 0.5 --threshold 5";
        let pipeline = parse_steps(steps).unwrap();
        let plan = pipeline.describe();
        assert_eq!(plan.steps.len(), 4);
    }
}
