mod app;
mod commands;

use app::{Cli, Commands, OutputFormat};
use clap::Parser;
use commands::CommandResult;
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
            CommandCap {
                name: "grayscale".into(),
                description: "Convert image to grayscale".into(),
            },
            CommandCap {
                name: "invert".into(),
                description: "Invert (negate) image colors".into(),
            },
            CommandCap {
                name: "brightness".into(),
                description: "Adjust image brightness".into(),
            },
            CommandCap {
                name: "contrast".into(),
                description: "Adjust image contrast".into(),
            },
            CommandCap {
                name: "hue-rotate".into(),
                description: "Rotate image hue".into(),
            },
            CommandCap {
                name: "blur".into(),
                description: "Apply Gaussian blur to an image".into(),
            },
            CommandCap {
                name: "sharpen".into(),
                description: "Sharpen an image using unsharp mask".into(),
            },
            CommandCap {
                name: "edge-detect".into(),
                description: "Detect edges in an image using Laplacian kernel".into(),
            },
            CommandCap {
                name: "emboss".into(),
                description: "Apply emboss effect to an image".into(),
            },
            CommandCap {
                name: "draw".into(),
                description: "Draw shapes (rect, circle, line) on an image".into(),
            },
            CommandCap {
                name: "overlay".into(),
                description: "Overlay (composite) one image on top of another".into(),
            },
            CommandCap {
                name: "trim".into(),
                description: "Trim (auto-crop) whitespace or similar-colored borders".into(),
            },
            CommandCap {
                name: "diff".into(),
                description: "Compare two images and visualize differences".into(),
            },
            CommandCap {
                name: "pipeline".into(),
                description: "Run multiple operations in a single read/write pipeline".into(),
            },
            CommandCap {
                name: "frames".into(),
                description: "Extract individual frames from an animated GIF".into(),
            },
            CommandCap {
                name: "animate".into(),
                description: "Assemble images into an animated GIF".into(),
            },
            CommandCap {
                name: "gif-speed".into(),
                description: "Change the speed of an animated GIF".into(),
            },
            CommandCap {
                name: "saturate".into(),
                description: "Adjust color saturation".into(),
            },
            CommandCap {
                name: "sepia".into(),
                description: "Apply sepia tone effect".into(),
            },
            CommandCap {
                name: "tint".into(),
                description: "Tint image with a color".into(),
            },
            CommandCap {
                name: "posterize".into(),
                description: "Reduce color levels (posterize)".into(),
            },
            CommandCap {
                name: "tilt-shift".into(),
                description: "Simulate tilt-shift (miniature/diorama) lens effect".into(),
            },
            CommandCap {
                name: "smart-crop".into(),
                description: "Automatically select the best crop region based on image content"
                    .into(),
            },
            CommandCap {
                name: "set-density".into(),
                description:
                    "Set image resolution/density (DPI/DPCM) metadata, optionally resampling pixels"
                        .into(),
            },
            CommandCap {
                name: "psd-info".into(),
                description: "Show PSD layer metadata".into(),
            },
            CommandCap {
                name: "psd-layers".into(),
                description: "Extract individual layers from a PSD file".into(),
            },
            CommandCap {
                name: "pdf-pages".into(),
                description: "Extract individual pages from a PDF file".into(),
            },
            CommandCap {
                name: "text".into(),
                description: "Draw text on an image (watermark, annotation)".into(),
            },
            CommandCap {
                name: "tiny".into(),
                description: "Smart image compression (like TinyPNG)".into(),
            },
            CommandCap {
                name: "batch".into(),
                description: "Process multiple files with glob patterns and parallel execution"
                    .into(),
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
            "--dpi <number>".into(),
            "--capabilities".into(),
        ],
    }
}

fn dispatch_schema(cli: &Cli, ctx: &app::RunContext) -> i32 {
    let schema = match &cli.command {
        Some(Commands::Info(_)) => Some(commands::info::schema()),
        Some(Commands::Convert(_)) => Some(commands::convert::schema()),
        Some(Commands::Resize(_)) => Some(commands::resize::schema()),
        Some(Commands::Crop(_)) => Some(commands::crop::schema()),
        Some(Commands::Rotate(_)) => Some(commands::rotate::schema()),
        Some(Commands::Flip(_)) => Some(commands::flip::schema()),
        Some(Commands::AutoOrient(_)) => Some(commands::orient::schema()),
        Some(Commands::Grayscale(_)) => Some(commands::grayscale::schema()),
        Some(Commands::Invert(_)) => Some(commands::invert::schema()),
        Some(Commands::Brightness(_)) => Some(commands::brightness::schema()),
        Some(Commands::Contrast(_)) => Some(commands::contrast::schema()),
        Some(Commands::HueRotate(_)) => Some(commands::hue_rotate::schema()),
        Some(Commands::Blur(_)) => Some(commands::blur::schema()),
        Some(Commands::Sharpen(_)) => Some(commands::sharpen::schema()),
        Some(Commands::EdgeDetect(_)) => Some(commands::edge_detect::schema()),
        Some(Commands::Emboss(_)) => Some(commands::emboss::schema()),
        Some(Commands::Draw(_)) => Some(commands::draw::schema()),
        Some(Commands::Overlay(_)) => Some(commands::overlay::schema()),
        Some(Commands::Trim(_)) => Some(commands::trim::schema()),
        Some(Commands::Diff(_)) => None,
        Some(Commands::Pipeline(_)) => None,
        Some(Commands::Frames(_)) => None,
        Some(Commands::Animate(_)) => None,
        Some(Commands::GifSpeed(_)) => None,
        Some(Commands::Saturate(_)) => Some(commands::saturate::schema()),
        Some(Commands::Sepia(_)) => Some(commands::sepia::schema()),
        Some(Commands::Tint(_)) => Some(commands::tint::schema()),
        Some(Commands::Posterize(_)) => Some(commands::posterize::schema()),
        Some(Commands::TiltShift(_)) => Some(commands::tilt_shift::schema()),
        Some(Commands::SmartCrop(_)) => Some(commands::smart_crop::schema()),
        Some(Commands::SetDensity(_)) => Some(commands::set_density::schema()),
        #[cfg(feature = "psd")]
        Some(Commands::PsdInfo(_)) => Some(commands::psd_info::schema()),
        #[cfg(feature = "psd")]
        Some(Commands::PsdLayers(_)) => Some(commands::psd_layers::schema()),
        #[cfg(feature = "pdf")]
        Some(Commands::PdfPages(_)) => Some(commands::pdf_pages::schema()),
        #[cfg(feature = "text")]
        Some(Commands::Text(_)) => Some(commands::text::schema()),
        #[cfg(feature = "tiny")]
        Some(Commands::Tiny(_)) => Some(commands::tiny::schema()),
        Some(Commands::Batch(_)) => None,
        None => None,
    };
    match schema {
        Some(s) => {
            ctx.print_json(&serde_json::to_value(&s).unwrap());
            0
        }
        None => {
            let err = panimg_core::error::PanimgError::InvalidArgument {
                message: "this command does not support --schema".into(),
                suggestion: "remove the --schema flag".into(),
            };
            ctx.print_error(&err)
        }
    }
}

fn dispatch_command(cli: &Cli, ctx: &app::RunContext) -> CommandResult {
    match &cli.command {
        Some(Commands::Info(args)) => commands::info::run(args, ctx),
        Some(Commands::Convert(args)) => commands::convert::run(args, ctx),
        Some(Commands::Resize(args)) => commands::resize::run(args, ctx),
        Some(Commands::Crop(args)) => commands::crop::run(args, ctx),
        Some(Commands::Rotate(args)) => commands::rotate::run(args, ctx),
        Some(Commands::Flip(args)) => commands::flip::run(args, ctx),
        Some(Commands::AutoOrient(args)) => commands::orient::run(args, ctx),
        Some(Commands::Grayscale(args)) => commands::grayscale::run(args, ctx),
        Some(Commands::Invert(args)) => commands::invert::run(args, ctx),
        Some(Commands::Brightness(args)) => commands::brightness::run(args, ctx),
        Some(Commands::Contrast(args)) => commands::contrast::run(args, ctx),
        Some(Commands::HueRotate(args)) => commands::hue_rotate::run(args, ctx),
        Some(Commands::Blur(args)) => commands::blur::run(args, ctx),
        Some(Commands::Sharpen(args)) => commands::sharpen::run(args, ctx),
        Some(Commands::EdgeDetect(args)) => commands::edge_detect::run(args, ctx),
        Some(Commands::Emboss(args)) => commands::emboss::run(args, ctx),
        Some(Commands::Draw(args)) => commands::draw::run(args, ctx),
        Some(Commands::Overlay(args)) => commands::overlay::run(args, ctx),
        Some(Commands::Trim(args)) => commands::trim::run(args, ctx),
        Some(Commands::Diff(args)) => commands::diff::run(args, ctx),
        Some(Commands::Pipeline(args)) => commands::pipeline::run(args, ctx),
        Some(Commands::Frames(args)) => commands::frames::run(args, ctx),
        Some(Commands::Animate(args)) => commands::animate::run(args, ctx),
        Some(Commands::GifSpeed(args)) => commands::gif_speed::run(args, ctx),
        Some(Commands::Saturate(args)) => commands::saturate::run(args, ctx),
        Some(Commands::Sepia(args)) => commands::sepia::run(args, ctx),
        Some(Commands::Tint(args)) => commands::tint::run(args, ctx),
        Some(Commands::Posterize(args)) => commands::posterize::run(args, ctx),
        Some(Commands::TiltShift(args)) => commands::tilt_shift::run(args, ctx),
        Some(Commands::SmartCrop(args)) => commands::smart_crop::run(args, ctx),
        Some(Commands::SetDensity(args)) => commands::set_density::run(args, ctx),
        #[cfg(feature = "psd")]
        Some(Commands::PsdInfo(args)) => commands::psd_info::run(args, ctx),
        #[cfg(feature = "psd")]
        Some(Commands::PsdLayers(args)) => commands::psd_layers::run(args, ctx),
        #[cfg(feature = "pdf")]
        Some(Commands::PdfPages(args)) => commands::pdf_pages::run(args, ctx),
        #[cfg(feature = "text")]
        Some(Commands::Text(args)) => commands::text::run(args, ctx),
        #[cfg(feature = "tiny")]
        Some(Commands::Tiny(args)) => commands::tiny::run(args, ctx),
        Some(Commands::Batch(args)) => commands::batch::run(args, ctx),
        None => {
            use clap::CommandFactory;
            Cli::command().print_help().ok();
            println!();
            Ok(0)
        }
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

    let ctx = app::RunContext::from_cli(&cli);

    let exit_code = if cli.schema {
        dispatch_schema(&cli, &ctx)
    } else {
        match dispatch_command(&cli, &ctx) {
            Ok(code) => code,
            Err(e) => ctx.print_error(&e),
        }
    };

    std::process::exit(exit_code);
}
