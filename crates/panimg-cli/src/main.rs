mod app;
mod commands;
mod output;

use app::{Cli, Commands, OutputFormat};
use clap::Parser;
use panimg_core::format::ImageFormat;
use serde::Serialize;

#[derive(Serialize)]
struct Capabilities {
    version: String,
    commands: Vec<CommandCap>,
    formats: Vec<FormatCap>,
    global_flags: Vec<String>,
}

#[derive(Serialize)]
struct CommandCap {
    name: String,
    description: String,
}

#[derive(Serialize)]
struct FormatCap {
    name: String,
    extension: String,
    can_decode: bool,
    can_encode: bool,
    mime_type: String,
}

fn capabilities() -> Capabilities {
    Capabilities {
        version: env!("CARGO_PKG_VERSION").to_string(),
        commands: vec![
            CommandCap {
                name: "info".into(),
                description: "Show image metadata and properties".into(),
            },
            CommandCap {
                name: "convert".into(),
                description: "Convert image between formats".into(),
            },
            CommandCap {
                name: "resize".into(),
                description: "Resize an image".into(),
            },
            CommandCap {
                name: "crop".into(),
                description: "Crop a rectangular region from an image".into(),
            },
            CommandCap {
                name: "rotate".into(),
                description: "Rotate an image by 90, 180, or 270 degrees".into(),
            },
            CommandCap {
                name: "flip".into(),
                description: "Flip (mirror) an image horizontally or vertically".into(),
            },
            CommandCap {
                name: "auto-orient".into(),
                description: "Auto-rotate image based on EXIF orientation tag".into(),
            },
        ],
        formats: ImageFormat::all()
            .iter()
            .map(|f| FormatCap {
                name: f.to_string(),
                extension: f.extension().to_string(),
                can_decode: f.can_decode(),
                can_encode: f.can_encode(),
                mime_type: f.mime_type().to_string(),
            })
            .collect(),
        global_flags: vec![
            "--format <human|json>".into(),
            "--dry-run".into(),
            "--schema".into(),
            "--capabilities".into(),
        ],
    }
}

fn main() {
    let cli = Cli::parse();

    // Handle --capabilities
    if cli.capabilities {
        let caps = capabilities();
        match cli.format {
            OutputFormat::Human => {
                println!("panimg v{}", caps.version);
                println!();
                println!("Commands:");
                for cmd in &caps.commands {
                    println!("  {:12} {}", cmd.name, cmd.description);
                }
                println!();
                println!("Supported formats:");
                for fmt in &caps.formats {
                    let decode = if fmt.can_decode { "decode" } else { "-" };
                    let encode = if fmt.can_encode { "encode" } else { "-" };
                    println!(
                        "  {:10} .{:5} {:6} {:6}  {}",
                        fmt.name, fmt.extension, decode, encode, fmt.mime_type
                    );
                }
                println!();
                println!("Global flags:");
                for flag in &caps.global_flags {
                    println!("  {flag}");
                }
            }
            OutputFormat::Json => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&caps).unwrap_or_else(|_| "{}".into())
                );
            }
        }
        std::process::exit(0);
    }

    let exit_code = match &cli.command {
        Some(Commands::Info(args)) => commands::info::run(args, cli.format, cli.schema),
        Some(Commands::Convert(args)) => {
            commands::convert::run(args, cli.format, cli.dry_run, cli.schema)
        }
        Some(Commands::Resize(args)) => {
            commands::resize::run(args, cli.format, cli.dry_run, cli.schema)
        }
        Some(Commands::Crop(args)) => {
            commands::crop::run(args, cli.format, cli.dry_run, cli.schema)
        }
        Some(Commands::Rotate(args)) => {
            commands::rotate::run(args, cli.format, cli.dry_run, cli.schema)
        }
        Some(Commands::Flip(args)) => {
            commands::flip::run(args, cli.format, cli.dry_run, cli.schema)
        }
        Some(Commands::AutoOrient(args)) => {
            commands::orient::run(args, cli.format, cli.dry_run, cli.schema)
        }
        None => {
            // No subcommand: show help
            use clap::CommandFactory;
            Cli::command().print_help().ok();
            println!();
            0
        }
    };

    std::process::exit(exit_code);
}
