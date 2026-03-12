use crate::error::{PanimgError, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Named position for overlay and text placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Position {
    Center,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Position {
    /// All valid position names, for use in schemas and help text.
    pub fn choices() -> &'static [&'static str] {
        &[
            "center",
            "top-left",
            "top-right",
            "bottom-left",
            "bottom-right",
        ]
    }
}

impl FromStr for Position {
    type Err = PanimgError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "center" => Ok(Self::Center),
            "top-left" => Ok(Self::TopLeft),
            "top-right" => Ok(Self::TopRight),
            "bottom-left" => Ok(Self::BottomLeft),
            "bottom-right" => Ok(Self::BottomRight),
            _ => Err(PanimgError::InvalidArgument {
                message: format!("unknown position: '{s}'"),
                suggestion: "use: center, top-left, top-right, bottom-left, bottom-right".into(),
            }),
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Center => write!(f, "center"),
            Self::TopLeft => write!(f, "top-left"),
            Self::TopRight => write!(f, "top-right"),
            Self::BottomLeft => write!(f, "bottom-left"),
            Self::BottomRight => write!(f, "bottom-right"),
        }
    }
}

/// Calculate x/y offset for named positions.
///
/// Returns `(i64, i64)` — enum is exhaustive so no invalid input is possible.
pub fn resolve_position(
    position: Position,
    base_w: u32,
    base_h: u32,
    overlay_w: u32,
    overlay_h: u32,
    margin: i64,
) -> (i64, i64) {
    match position {
        Position::Center => (
            (base_w as i64 - overlay_w as i64) / 2,
            (base_h as i64 - overlay_h as i64) / 2,
        ),
        Position::TopLeft => (margin, margin),
        Position::TopRight => (base_w as i64 - overlay_w as i64 - margin, margin),
        Position::BottomLeft => (margin, base_h as i64 - overlay_h as i64 - margin),
        Position::BottomRight => (
            base_w as i64 - overlay_w as i64 - margin,
            base_h as i64 - overlay_h as i64 - margin,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_roundtrip() {
        for &name in Position::choices() {
            let pos: Position = name.parse().unwrap();
            assert_eq!(pos.to_string(), name);
        }
    }

    #[test]
    fn from_str_unknown() {
        assert!("middle".parse::<Position>().is_err());
    }

    #[test]
    fn resolve_center() {
        let (x, y) = resolve_position(Position::Center, 100, 100, 20, 20, 0);
        assert_eq!(x, 40);
        assert_eq!(y, 40);
    }

    #[test]
    fn resolve_corners() {
        let margin = 10;
        let (x, y) = resolve_position(Position::TopLeft, 100, 100, 20, 20, margin);
        assert_eq!(x, 10);
        assert_eq!(y, 10);

        let (x, y) = resolve_position(Position::BottomRight, 100, 100, 20, 20, margin);
        assert_eq!(x, 70);
        assert_eq!(y, 70);
    }

    #[test]
    fn serde_roundtrip() {
        let pos = Position::BottomRight;
        let json = serde_json::to_string(&pos).unwrap();
        assert_eq!(json, "\"bottom-right\"");
        let parsed: Position = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, pos);
    }
}
