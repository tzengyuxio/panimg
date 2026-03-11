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

    /// Convert image to grayscale
    Grayscale(GrayscaleArgs),

    /// Invert (negate) image colors
    Invert(InvertArgs),

    /// Adjust image brightness
    Brightness(BrightnessArgs),

    /// Adjust image contrast
    Contrast(ContrastArgs),

    /// Rotate image hue
    HueRotate(HueRotateArgs),

    /// Apply Gaussian blur
    Blur(BlurArgs),

    /// Sharpen an image (unsharp mask)
    Sharpen(SharpenArgs),

    /// Detect edges in an image
    EdgeDetect(EdgeDetectArgs),

    /// Apply emboss effect
    Emboss(EmbossArgs),

    /// Overlay (composite) one image on top of another
    Overlay(OverlayArgs),

    /// Process multiple files in batch with glob patterns
    Batch(BatchArgs),
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

#[derive(Parser)]
pub struct GrayscaleArgs {
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

#[derive(Parser)]
pub struct InvertArgs {
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

#[derive(Parser)]
pub struct BrightnessArgs {
    /// Input image file
    pub input: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Brightness adjustment value (-100 to 100)
    #[arg(long)]
    pub value: Option<i32>,

    /// Output quality (1-100, for lossy formats)
    #[arg(long)]
    pub quality: Option<u8>,

    /// Strip metadata from output
    #[arg(long)]
    pub strip: bool,
}

#[derive(Parser)]
pub struct ContrastArgs {
    /// Input image file
    pub input: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Contrast adjustment value (-100 to 100)
    #[arg(long)]
    pub value: Option<f32>,

    /// Output quality (1-100, for lossy formats)
    #[arg(long)]
    pub quality: Option<u8>,

    /// Strip metadata from output
    #[arg(long)]
    pub strip: bool,
}

#[derive(Parser)]
pub struct HueRotateArgs {
    /// Input image file
    pub input: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Hue rotation in degrees (-360 to 360)
    #[arg(long)]
    pub degrees: Option<i32>,

    /// Output quality (1-100, for lossy formats)
    #[arg(long)]
    pub quality: Option<u8>,

    /// Strip metadata from output
    #[arg(long)]
    pub strip: bool,
}

#[derive(Parser)]
pub struct BlurArgs {
    /// Input image file
    pub input: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Blur sigma (radius). Higher values = more blur
    #[arg(long)]
    pub sigma: Option<f32>,

    /// Output quality (1-100, for lossy formats)
    #[arg(long)]
    pub quality: Option<u8>,

    /// Strip metadata from output
    #[arg(long)]
    pub strip: bool,
}

#[derive(Parser)]
pub struct SharpenArgs {
    /// Input image file
    pub input: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Unsharp mask sigma. Controls effect radius
    #[arg(long)]
    pub sigma: Option<f32>,

    /// Only sharpen differences above this threshold (default: 0)
    #[arg(long)]
    pub threshold: Option<i32>,

    /// Output quality (1-100, for lossy formats)
    #[arg(long)]
    pub quality: Option<u8>,

    /// Strip metadata from output
    #[arg(long)]
    pub strip: bool,
}

#[derive(Parser)]
pub struct EdgeDetectArgs {
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

#[derive(Parser)]
pub struct EmbossArgs {
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

#[derive(Parser)]
pub struct OverlayArgs {
    /// Base image file
    pub input: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Overlay image path
    #[arg(long)]
    pub layer: Option<String>,

    /// X offset from left edge (can be negative)
    #[arg(long)]
    pub x: Option<i64>,

    /// Y offset from top edge (can be negative)
    #[arg(long)]
    pub y: Option<i64>,

    /// Opacity of the overlay (0.0 = transparent, 1.0 = opaque)
    #[arg(long)]
    pub opacity: Option<f32>,

    /// Named position: center, top-left, top-right, bottom-left, bottom-right
    #[arg(long)]
    pub position: Option<String>,

    /// Margin in pixels for named positions (default: 10)
    #[arg(long)]
    pub margin: Option<i64>,

    /// Tile the overlay across the entire image
    #[arg(long)]
    pub tile: bool,

    /// Spacing between tiles in pixels
    #[arg(long)]
    pub spacing: Option<u32>,

    /// Output quality (1-100, for lossy formats)
    #[arg(long)]
    pub quality: Option<u8>,

    /// Strip metadata from output
    #[arg(long)]
    pub strip: bool,
}

#[derive(Parser)]
pub struct BatchArgs {
    /// Operation to apply (convert, resize, crop, rotate, flip, auto-orient, grayscale, invert, brightness, contrast, hue-rotate, blur, sharpen, edge-detect, emboss)
    pub operation: String,

    /// Glob pattern for input files (e.g. "photos/*.png")
    pub pattern: String,

    /// Output directory
    #[arg(long)]
    pub output_dir: Option<String>,

    /// Output path template with placeholders: {stem}, {name}, {ext}, {dir}
    #[arg(long)]
    pub output_template: Option<String>,

    /// Target format for convert (e.g. webp, jpg)
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

    // --- Operation-specific args ---
    /// Target width (for resize)
    #[arg(long)]
    pub width: Option<u32>,

    /// Target height (for resize)
    #[arg(long)]
    pub height: Option<u32>,

    /// Fit mode (for resize): contain, cover, fill, inside, outside
    #[arg(long)]
    pub fit: Option<String>,

    /// Resize filter: lanczos3, catmull-rom, nearest, linear
    #[arg(long)]
    pub filter: Option<String>,

    /// Left offset (for crop)
    #[arg(long)]
    pub x: Option<u32>,

    /// Top offset (for crop)
    #[arg(long)]
    pub y: Option<u32>,

    /// Crop width
    #[arg(long)]
    pub crop_width: Option<u32>,

    /// Crop height
    #[arg(long)]
    pub crop_height: Option<u32>,

    /// Rotation angle (for rotate): 90, 180, 270, left, right
    #[arg(long)]
    pub angle: Option<String>,

    /// Flip direction: horizontal, vertical
    #[arg(long)]
    pub direction: Option<String>,

    /// Brightness adjustment value (for brightness): -100 to 100
    #[arg(long = "value")]
    pub brightness_value: Option<i32>,

    /// Contrast adjustment value (for contrast): -100 to 100
    #[arg(long)]
    pub contrast_value: Option<f32>,

    /// Hue rotation degrees (for hue-rotate): -360 to 360
    #[arg(long)]
    pub degrees: Option<i32>,

    /// Blur/sharpen sigma (for blur, sharpen)
    #[arg(long)]
    pub sigma: Option<f32>,

    /// Sharpen threshold (for sharpen)
    #[arg(long)]
    pub threshold: Option<i32>,
}
