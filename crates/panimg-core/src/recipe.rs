#[cfg(feature = "text")]
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
#[cfg(feature = "text")]
use crate::ops::text::DrawTextOp;
use crate::ops::trim::TrimOp;
use crate::ops::Operation;
use crate::pipeline::Pipeline;
use serde::Deserialize;

/// A single step in a recipe, parsed from JSON.
#[derive(Debug, Deserialize)]
pub struct RecipeStep {
    pub op: String,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub fit: Option<String>,
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default)]
    pub x: Option<u32>,
    #[serde(default)]
    pub y: Option<u32>,
    #[serde(default)]
    pub angle: Option<String>,
    #[serde(default)]
    pub direction: Option<String>,
    #[serde(default)]
    pub value: Option<i32>,
    #[serde(default)]
    pub contrast_value: Option<f32>,
    #[serde(default)]
    pub degrees: Option<i32>,
    #[serde(default)]
    pub sigma: Option<f32>,
    #[serde(default)]
    pub threshold: Option<i32>,
    #[serde(default)]
    pub tolerance: Option<u8>,
    #[serde(default)]
    pub factor: Option<f32>,
    #[serde(default)]
    pub intensity: Option<f32>,
    #[serde(default)]
    pub levels: Option<u8>,
    // Text rendering fields
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub font: Option<String>,
    #[serde(default)]
    pub size: Option<f32>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub position: Option<String>,
    #[serde(default)]
    pub margin: Option<u32>,
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
fn parse_single_step(step: &str) -> Result<Box<dyn Operation>> {
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
            Ok(Box::new(RotateOp::new(RotateAngle::parse(&angle_str)?)))
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
            suggestion: "supported: grayscale, invert, blur, sharpen, brightness, contrast, hue-rotate, resize, crop, rotate, flip, edge-detect, emboss, trim, saturate, sepia, posterize, text".into(),
        }),
    }
}

/// Build an operation from a JSON recipe step.
fn build_op_from_recipe_step(step: &RecipeStep) -> Result<Box<dyn Operation>> {
    match step.op.as_str() {
        "grayscale" => Ok(Box::new(GrayscaleOp::new())),
        "invert" => Ok(Box::new(InvertOp::new())),
        "edge-detect" => Ok(Box::new(EdgeDetectOp::new())),
        "emboss" => Ok(Box::new(EmbossOp::new())),
        "blur" => {
            let sigma = step.sigma.ok_or_else(|| PanimgError::InvalidArgument {
                message: "blur step requires \"sigma\"".into(),
                suggestion: "{\"op\": \"blur\", \"sigma\": 2.0}".into(),
            })?;
            Ok(Box::new(BlurOp::new(sigma)?))
        }
        "sharpen" => {
            let sigma = step.sigma.ok_or_else(|| PanimgError::InvalidArgument {
                message: "sharpen step requires \"sigma\"".into(),
                suggestion: "{\"op\": \"sharpen\", \"sigma\": 1.5}".into(),
            })?;
            let threshold = step.threshold.unwrap_or(0);
            Ok(Box::new(SharpenOp::new(sigma, threshold)?))
        }
        "brightness" => {
            let value = step.value.ok_or_else(|| PanimgError::InvalidArgument {
                message: "brightness step requires \"value\"".into(),
                suggestion: "{\"op\": \"brightness\", \"value\": 20}".into(),
            })?;
            Ok(Box::new(BrightnessOp::new(value)?))
        }
        "contrast" => {
            let value = step.contrast_value.ok_or_else(|| PanimgError::InvalidArgument {
                message: "contrast step requires \"contrast_value\"".into(),
                suggestion: "{\"op\": \"contrast\", \"contrast_value\": 1.5}".into(),
            })?;
            Ok(Box::new(ContrastOp::new(value)?))
        }
        "hue-rotate" => {
            let degrees = step.degrees.ok_or_else(|| PanimgError::InvalidArgument {
                message: "hue-rotate step requires \"degrees\"".into(),
                suggestion: "{\"op\": \"hue-rotate\", \"degrees\": 90}".into(),
            })?;
            Ok(Box::new(HueRotateOp::new(degrees)?))
        }
        "resize" => {
            let fit = step
                .fit
                .as_deref()
                .map(FitMode::parse)
                .transpose()?
                .unwrap_or(FitMode::Contain);
            let filter = step
                .filter
                .as_deref()
                .map(ResizeFilter::parse)
                .transpose()?
                .unwrap_or(ResizeFilter::Lanczos3);
            Ok(Box::new(ResizeOp::new(step.width, step.height, fit, filter)?))
        }
        "crop" => {
            let x = step.x.unwrap_or(0);
            let y = step.y.unwrap_or(0);
            let width = step.width.ok_or_else(|| PanimgError::InvalidArgument {
                message: "crop step requires \"width\"".into(),
                suggestion: "{\"op\": \"crop\", \"x\": 0, \"y\": 0, \"width\": 100, \"height\": 100}".into(),
            })?;
            let height = step.height.ok_or_else(|| PanimgError::InvalidArgument {
                message: "crop step requires \"height\"".into(),
                suggestion: "{\"op\": \"crop\", \"x\": 0, \"y\": 0, \"width\": 100, \"height\": 100}".into(),
            })?;
            Ok(Box::new(CropOp::new(x, y, width, height)?))
        }
        "rotate" => {
            let angle_str = step.angle.as_deref().ok_or_else(|| PanimgError::InvalidArgument {
                message: "rotate step requires \"angle\"".into(),
                suggestion: "{\"op\": \"rotate\", \"angle\": \"90\"}".into(),
            })?;
            Ok(Box::new(RotateOp::new(RotateAngle::parse(angle_str)?)))
        }
        "flip" => {
            let direction = step.direction.as_deref().ok_or_else(|| PanimgError::InvalidArgument {
                message: "flip step requires \"direction\"".into(),
                suggestion: "{\"op\": \"flip\", \"direction\": \"horizontal\"}".into(),
            })?;
            Ok(Box::new(FlipOp::new(FlipDirection::parse(direction)?)))
        }
        "trim" => {
            let tolerance = step.tolerance.unwrap_or(10);
            Ok(Box::new(TrimOp::new(tolerance)?))
        }
        "saturate" => {
            let factor = step.factor.ok_or_else(|| PanimgError::InvalidArgument {
                message: "saturate step requires \"factor\"".into(),
                suggestion: "{\"op\": \"saturate\", \"factor\": 1.5}".into(),
            })?;
            Ok(Box::new(SaturateOp::new(factor)?))
        }
        "sepia" => {
            let intensity = step.intensity.unwrap_or(1.0);
            Ok(Box::new(SepiaOp::new(intensity)?))
        }
        "posterize" => {
            let levels = step.levels.unwrap_or(4);
            Ok(Box::new(PosterizeOp::new(levels)?))
        }
        #[cfg(feature = "text")]
        "text" => {
            let content = step.content.clone().ok_or_else(|| PanimgError::InvalidArgument {
                message: "text step requires \"content\"".into(),
                suggestion: "{\"op\": \"text\", \"content\": \"Hello World\"}".into(),
            })?;
            let size = step.size.unwrap_or(24.0);
            let color_str = step.color.as_deref().unwrap_or("white");
            let color = parse_color(color_str)?;
            let x = step.x.map(|v| v as i32);
            let y = step.y.map(|v| v as i32);
            let position = step
                .position
                .as_deref()
                .map(|s| s.parse::<Position>())
                .transpose()?;
            let margin = step.margin.unwrap_or(10);
            Ok(Box::new(DrawTextOp::new(
                content,
                step.font.as_deref(),
                size,
                color,
                x,
                y,
                position,
                margin,
            )?))
        }
        _ => Err(PanimgError::InvalidArgument {
            message: format!("unknown recipe operation: '{}'", step.op),
            suggestion: "supported: grayscale, invert, blur, sharpen, brightness, contrast, hue-rotate, resize, crop, rotate, flip, edge-detect, emboss, trim, saturate, sepia, posterize, text".into(),
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
