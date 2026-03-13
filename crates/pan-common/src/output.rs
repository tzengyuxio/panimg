use crate::error::StructuredError;
use serde::Serialize;

/// Output format for CLI tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum OutputFormat {
    Human,
    Json,
}

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
    // Value serialization is infallible — unwrap is safe here.
    println!("{}", serde_json::to_string_pretty(value).unwrap());
}

/// Print an error in the appropriate format and return the exit code.
pub fn print_error<E: StructuredError>(format: OutputFormat, err: &E) -> i32 {
    let exit_code: u8 = err.exit_code().into();
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
                serde_json::to_string(&serde_json::json!({
                    "error": "serialization_failed",
                    "message": err.to_string()
                }))
                .unwrap_or_default()
            });
            eprintln!("{json}");
        }
    }
    exit_code as i32
}
