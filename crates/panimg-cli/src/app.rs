use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "panimg",
    version,
    about = "Next-generation image processing CLI",
    long_about = "A modern, AI-agent-friendly image processing tool with structured output, \
                  dry-run support, and consistent syntax."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Output format
    #[arg(long, global = true, default_value = "human")]
    pub format: OutputFormat,

    /// Preview operations without executing
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Show command parameter schema as JSON
    #[arg(long, global = true)]
    pub schema: bool,

    /// List all supported commands, formats, and features
    #[arg(long)]
    pub capabilities: bool,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Human,
    Json,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show image metadata and properties
    Info(InfoArgs),

    /// Convert image between formats
    Convert(ConvertArgs),

    /// Resize an image
    Resize(ResizeArgs),

    /// Crop a rectangular region from an image
    Crop(CropArgs),

    /// Rotate an image by 90, 180, or 270 degrees
    Rotate(RotateArgs),

    /// Flip (mirror) an image horizontally or vertically
    Flip(FlipArgs),

    /// Auto-rotate image based on EXIF orientation tag
    AutoOrient(AutoOrientArgs),
}

#[derive(Parser)]
pub struct InfoArgs {
    /// Input image file
    pub input: Option<String>,

    /// Comma-separated list of fields to show
    #[arg(long)]
    pub fields: Option<String>,
}

#[derive(Parser)]
pub struct ConvertArgs {
    /// Input image file
    pub input: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Target format (inferred from output extension if not set)
    #[arg(long)]
    pub to: Option<String>,

    /// Output quality (1-100, for lossy formats)
    #[arg(long)]
    pub quality: Option<u8>,

    /// Strip metadata from output
    #[arg(long)]
    pub strip: bool,

    /// Overwrite output if it exists
    #[arg(long)]
    pub overwrite: bool,

    /// Skip if output already exists
    #[arg(long)]
    pub skip_existing: bool,
}

#[derive(Parser)]
pub struct ResizeArgs {
    /// Input image file
    pub input: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Target width in pixels
    #[arg(long)]
    pub width: Option<u32>,

    /// Target height in pixels
    #[arg(long)]
    pub height: Option<u32>,

    /// Fit mode: contain, cover, fill, inside, outside
    #[arg(long, default_value = "contain")]
    pub fit: String,

    /// Resize filter: lanczos3, catmull-rom, nearest, linear
    #[arg(long, default_value = "lanczos3")]
    pub filter: String,

    /// Output quality (1-100, for lossy formats)
    #[arg(long)]
    pub quality: Option<u8>,

    /// Strip metadata from output
    #[arg(long)]
    pub strip: bool,
}

#[derive(Parser)]
pub struct CropArgs {
    /// Input image file
    pub input: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Left offset in pixels
    #[arg(long, default_value = "0")]
    pub x: u32,

    /// Top offset in pixels
    #[arg(long, default_value = "0")]
    pub y: u32,

    /// Crop width in pixels
    #[arg(long)]
    pub width: Option<u32>,

    /// Crop height in pixels
    #[arg(long)]
    pub height: Option<u32>,

    /// Output quality (1-100, for lossy formats)
    #[arg(long)]
    pub quality: Option<u8>,

    /// Strip metadata from output
    #[arg(long)]
    pub strip: bool,
}

#[derive(Parser)]
pub struct RotateArgs {
    /// Input image file
    pub input: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Rotation angle: 90, 180, 270, left, right
    #[arg(long)]
    pub angle: Option<String>,

    /// Output quality (1-100, for lossy formats)
    #[arg(long)]
    pub quality: Option<u8>,

    /// Strip metadata from output
    #[arg(long)]
    pub strip: bool,
}

#[derive(Parser)]
pub struct FlipArgs {
    /// Input image file
    pub input: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Flip direction: horizontal (h), vertical (v)
    #[arg(long)]
    pub direction: Option<String>,

    /// Output quality (1-100, for lossy formats)
    #[arg(long)]
    pub quality: Option<u8>,

    /// Strip metadata from output
    #[arg(long)]
    pub strip: bool,
}

#[derive(Parser)]
pub struct AutoOrientArgs {
    /// Input image file
    pub input: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Output quality (1-100, for lossy formats)
    #[arg(long)]
    pub quality: Option<u8>,

    /// Strip metadata from output
    #[arg(long)]
    pub strip: bool,
}
