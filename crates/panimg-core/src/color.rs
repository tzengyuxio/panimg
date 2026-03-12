use crate::error::{PanimgError, Result};
use image::Rgba;

/// Parse a color string. Supports:
/// - Hex: #FF0000, #FF0000FF
/// - RGB: 255,0,0
/// - RGBA: 255,0,0,128
/// - Named: red, green, blue, white, black, yellow, cyan, magenta
pub fn parse_color(s: &str) -> Result<Rgba<u8>> {
    let s = s.trim();

    // Named colors
    match s.to_lowercase().as_str() {
        "red" => return Ok(Rgba([255, 0, 0, 255])),
        "green" => return Ok(Rgba([0, 255, 0, 255])),
        "blue" => return Ok(Rgba([0, 0, 255, 255])),
        "white" => return Ok(Rgba([255, 255, 255, 255])),
        "black" => return Ok(Rgba([0, 0, 0, 255])),
        "yellow" => return Ok(Rgba([255, 255, 0, 255])),
        "cyan" => return Ok(Rgba([0, 255, 255, 255])),
        "magenta" => return Ok(Rgba([255, 0, 255, 255])),
        _ => {}
    }

    // Hex format
    if let Some(hex) = s.strip_prefix('#') {
        return match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| color_err(s))?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| color_err(s))?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| color_err(s))?;
                Ok(Rgba([r, g, b, 255]))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| color_err(s))?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| color_err(s))?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| color_err(s))?;
                let a = u8::from_str_radix(&hex[6..8], 16).map_err(|_| color_err(s))?;
                Ok(Rgba([r, g, b, a]))
            }
            _ => Err(color_err(s)),
        };
    }

    // CSV format: R,G,B or R,G,B,A
    let parts: Vec<&str> = s.split(',').collect();
    match parts.len() {
        3 => {
            let r: u8 = parts[0].trim().parse().map_err(|_| color_err(s))?;
            let g: u8 = parts[1].trim().parse().map_err(|_| color_err(s))?;
            let b: u8 = parts[2].trim().parse().map_err(|_| color_err(s))?;
            Ok(Rgba([r, g, b, 255]))
        }
        4 => {
            let r: u8 = parts[0].trim().parse().map_err(|_| color_err(s))?;
            let g: u8 = parts[1].trim().parse().map_err(|_| color_err(s))?;
            let b: u8 = parts[2].trim().parse().map_err(|_| color_err(s))?;
            let a: u8 = parts[3].trim().parse().map_err(|_| color_err(s))?;
            Ok(Rgba([r, g, b, a]))
        }
        _ => Err(color_err(s)),
    }
}

fn color_err(s: &str) -> PanimgError {
    PanimgError::InvalidArgument {
        message: format!("invalid color: '{s}'"),
        suggestion: "use hex (#FF0000), RGB (255,0,0), or named (red, blue, etc.)".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_color() {
        let c = parse_color("#FF0000").unwrap();
        assert_eq!(c, Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn parse_hex_color_with_alpha() {
        let c = parse_color("#00FF0080").unwrap();
        assert_eq!(c, Rgba([0, 255, 0, 128]));
    }

    #[test]
    fn parse_rgb_color() {
        let c = parse_color("128,64,32").unwrap();
        assert_eq!(c, Rgba([128, 64, 32, 255]));
    }

    #[test]
    fn parse_rgba_color() {
        let c = parse_color("128,64,32,200").unwrap();
        assert_eq!(c, Rgba([128, 64, 32, 200]));
    }

    #[test]
    fn parse_named_color() {
        assert_eq!(parse_color("red").unwrap(), Rgba([255, 0, 0, 255]));
        assert_eq!(parse_color("blue").unwrap(), Rgba([0, 0, 255, 255]));
    }

    #[test]
    fn parse_invalid_color() {
        assert!(parse_color("invalid").is_err());
        assert!(parse_color("#GG0000").is_err());
    }
}
