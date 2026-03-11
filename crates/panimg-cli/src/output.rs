use crate::app::OutputFormat;
use panimg_core::error::PanimgError;
use serde::Serialize;

/// Print a value as either human text or JSON.
pub fn print_output<T: Serialize>(format: OutputFormat, human_text: &str, value: &T) {
    match format {
        OutputFormat::Human => println!("{human_text}"),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".into())
            );
        }
    }
}

/// Print a JSON value directly.
pub fn print_json(value: &serde_json::Value) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".into())
    );
}

/// Print an error in the appropriate format and return the exit code.
pub fn print_error(format: OutputFormat, err: &PanimgError) -> i32 {
    let exit_code = err.exit_code() as u8;
    match format {
        OutputFormat::Human => {
            eprintln!("error: {err}");
            let suggestion = err.suggestion();
            if !suggestion.is_empty() {
                eprintln!("  hint: {suggestion}");
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(err).unwrap_or_else(|_| {
                format!(r#"{{"error": "serialization_failed", "message": "{err}"}}"#)
            });
            eprintln!("{json}");
        }
    }
    exit_code as i32
}
