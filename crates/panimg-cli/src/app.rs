use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "panimg",
    version,
    about = "The Swiss Army knife of image processing — built for humans and AI agents alike.",
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

    /// Rotate an image by any angle (90/180/270 use fast lossless path)
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

    /// Draw shapes (rect, circle, line) on an image
    Draw(DrawArgs),

    /// Overlay (composite) one image on top of another
    Overlay(OverlayArgs),

    /// Trim (auto-crop) whitespace or similar-colored borders
    Trim(TrimArgs),

    /// Compare two images and visualize differences
    Diff(DiffArgs),

    /// Run multiple operations in a single read/write pipeline
    Pipeline(PipelineArgs),

    /// Extract individual frames from an animated GIF
    Frames(FramesArgs),

    /// Assemble images into an animated GIF
    Animate(AnimateArgs),

    /// Change the speed of an animated GIF
    GifSpeed(GifSpeedArgs),

    /// Adjust color saturation
    Saturate(SaturateArgs),

    /// Apply sepia tone effect
    Sepia(SepiaArgs),

    /// Tint image with a color
    Tint(TintArgs),

    /// Reduce color levels (posterize)
    Posterize(PosterizeArgs),

    /// Simulate tilt-shift (miniature/diorama) lens effect
    TiltShift(TiltShiftArgs),

    /// Automatically select the best crop region based on image content
    SmartCrop(SmartCropArgs),

    /// Draw text on an image (watermark, annotation)
    #[cfg(feature = "text")]
    Text(TextArgs),

    /// Smart image compression (like TinyPNG)
    #[cfg(feature = "tiny")]
    Tiny(TinyArgs),

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

    /// Rotation angle: 90, 180, 270, left, right, or any numeric angle (e.g. 45, 30.5)
    #[arg(long)]
    pub angle: Option<String>,

    /// Background fill color for arbitrary-angle rotation (hex, RGB, or named).
    /// Default: transparent for RGBA output, white for RGB output.
    #[arg(long)]
    pub background: Option<String>,

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
pub struct DrawArgs {
    /// Input image file
    pub input: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Shape type: rect, circle, line
    #[arg(long)]
    pub shape: Option<String>,

    /// Color: hex (#FF0000), RGB (255,0,0), or named (red, blue, etc.)
    #[arg(long)]
    pub color: Option<String>,

    /// Fill the shape (default: outlined)
    #[arg(long)]
    pub fill: bool,

    /// Line/border thickness in pixels
    #[arg(long)]
    pub thickness: Option<u32>,

    /// X position (rect/line start)
    #[arg(long)]
    pub x: Option<i32>,

    /// Y position (rect/line start)
    #[arg(long)]
    pub y: Option<i32>,

    /// Width (for rect)
    #[arg(long)]
    pub width: Option<u32>,

    /// Height (for rect)
    #[arg(long)]
    pub height: Option<u32>,

    /// Circle center X
    #[arg(long)]
    pub cx: Option<i32>,

    /// Circle center Y
    #[arg(long)]
    pub cy: Option<i32>,

    /// Circle radius
    #[arg(long)]
    pub radius: Option<u32>,

    /// Line end X
    #[arg(long)]
    pub x1: Option<i32>,

    /// Line end Y
    #[arg(long)]
    pub y1: Option<i32>,

    /// Line end X
    #[arg(long)]
    pub x2: Option<i32>,

    /// Line end Y
    #[arg(long)]
    pub y2: Option<i32>,

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
pub struct TrimArgs {
    /// Input image file
    pub input: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Color distance threshold for background detection (0-255, default: 10)
    #[arg(long)]
    pub tolerance: Option<u8>,

    /// Output quality (1-100, for lossy formats)
    #[arg(long)]
    pub quality: Option<u8>,

    /// Strip metadata from output
    #[arg(long)]
    pub strip: bool,
}

#[derive(Parser)]
pub struct DiffArgs {
    /// First input image
    pub input_a: Option<String>,

    /// Second input image
    pub input_b: Option<String>,

    /// Output diff visualization image
    #[arg(short, long)]
    pub output: Option<String>,

    /// Color channel difference threshold (0-255, default: 0)
    #[arg(long)]
    pub threshold: Option<u8>,

    /// Output quality (1-100, for lossy formats)
    #[arg(long)]
    pub quality: Option<u8>,
}

#[derive(Parser)]
pub struct PipelineArgs {
    /// Input image file
    pub input: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Pipeline steps (pipe-separated), e.g. "resize --width 800 | blur --sigma 2 | grayscale"
    #[arg(long)]
    pub steps: Option<String>,

    /// Path to a JSON recipe file
    #[arg(long)]
    pub recipe: Option<String>,

    /// Output quality (1-100, for lossy formats)
    #[arg(long)]
    pub quality: Option<u8>,

    /// Strip metadata from output
    #[arg(long)]
    pub strip: bool,
}

#[derive(Parser)]
pub struct FramesArgs {
    /// Input animated GIF file
    pub input: Option<String>,

    /// Output directory for extracted frames
    #[arg(long)]
    pub output_dir: Option<String>,

    /// Output format for frames (e.g. png, jpg)
    #[arg(long)]
    pub frame_format: Option<String>,

    /// Filename prefix for frames (default: "frame")
    #[arg(long)]
    pub prefix: Option<String>,
}

#[derive(Parser)]
pub struct AnimateArgs {
    /// Glob pattern for input images (e.g. "frames/*.png")
    pub pattern: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output GIF file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Delay between frames in milliseconds (default: 100)
    #[arg(long)]
    pub delay: Option<u32>,

    /// Do not loop the animation
    #[arg(long)]
    pub no_repeat: bool,
}

#[derive(Parser)]
pub struct GifSpeedArgs {
    /// Input animated GIF file
    pub input: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output GIF file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Speed multiplier (2.0 = 2x faster, 0.5 = half speed)
    #[arg(long)]
    pub speed: Option<f32>,
}

#[derive(Parser)]
pub struct SaturateArgs {
    /// Input image file
    pub input: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Saturation factor (0.0=grayscale, 1.0=unchanged, 2.0=double)
    #[arg(long)]
    pub factor: Option<f32>,

    /// Output quality (1-100, for lossy formats)
    #[arg(long)]
    pub quality: Option<u8>,

    /// Strip metadata from output
    #[arg(long)]
    pub strip: bool,
}

#[derive(Parser)]
pub struct SepiaArgs {
    /// Input image file
    pub input: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Sepia intensity (0.0-1.0, default: 1.0)
    #[arg(long)]
    pub intensity: Option<f32>,

    /// Output quality (1-100, for lossy formats)
    #[arg(long)]
    pub quality: Option<u8>,

    /// Strip metadata from output
    #[arg(long)]
    pub strip: bool,
}

#[derive(Parser)]
pub struct TintArgs {
    /// Input image file
    pub input: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Tint color: hex (#FF0000), RGB (255,0,0), or named (red, blue, etc.)
    #[arg(long)]
    pub color: Option<String>,

    /// Tint strength (0.0-1.0, default: 0.5)
    #[arg(long)]
    pub strength: Option<f32>,

    /// Output quality (1-100, for lossy formats)
    #[arg(long)]
    pub quality: Option<u8>,

    /// Strip metadata from output
    #[arg(long)]
    pub strip: bool,
}

#[derive(Parser)]
pub struct PosterizeArgs {
    /// Input image file
    pub input: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Number of color levels per channel (2-256, default: 4)
    #[arg(long)]
    pub levels: Option<u8>,

    /// Output quality (1-100, for lossy formats)
    #[arg(long)]
    pub quality: Option<u8>,

    /// Strip metadata from output
    #[arg(long)]
    pub strip: bool,
}

#[derive(Parser)]
pub struct TiltShiftArgs {
    /// Input image file
    pub input: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Out-of-focus blur strength (default: 8.0)
    #[arg(long)]
    pub sigma: Option<f32>,

    /// Vertical center of the focus band (0=top, 1=bottom, default: 0.5)
    #[arg(long)]
    pub focus_position: Option<f32>,

    /// Height of the focus band as fraction of image height (default: 0.15)
    #[arg(long)]
    pub focus_width: Option<f32>,

    /// Transition zone width as fraction of image height (default: 0.2)
    #[arg(long)]
    pub transition: Option<f32>,

    /// Saturation multiplier (>1 enhances miniature look, default: 1.0)
    #[arg(long)]
    pub saturation: Option<f32>,

    /// Output quality (1-100, for lossy formats)
    #[arg(long)]
    pub quality: Option<u8>,

    /// Strip metadata from output
    #[arg(long)]
    pub strip: bool,
}

#[derive(Parser)]
pub struct SmartCropArgs {
    /// Input image file
    pub input: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Crop width in pixels
    #[arg(long)]
    pub width: Option<u32>,

    /// Crop height in pixels
    #[arg(long)]
    pub height: Option<u32>,

    /// Scoring strategy: entropy or attention (default: entropy)
    #[arg(long)]
    pub strategy: Option<String>,

    /// Search step size in pixels (default: auto)
    #[arg(long)]
    pub step: Option<u32>,

    /// Output quality (1-100, for lossy formats)
    #[arg(long)]
    pub quality: Option<u8>,

    /// Strip metadata from output
    #[arg(long)]
    pub strip: bool,
}

#[cfg(feature = "text")]
#[derive(Parser)]
pub struct TextArgs {
    /// Input image file
    pub input: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Text content to draw
    #[arg(long)]
    pub content: Option<String>,

    /// TTF/OTF font file path (uses embedded DejaVu Sans if omitted)
    #[arg(long)]
    pub font: Option<String>,

    /// Font size in pixels (default: 24)
    #[arg(long)]
    pub size: Option<f32>,

    /// Text color: hex (#FFFFFF), RGB (255,255,255), or named (white, red, etc.)
    #[arg(long)]
    pub color: Option<String>,

    /// Absolute X position (overrides --position)
    #[arg(long)]
    pub x: Option<i32>,

    /// Absolute Y position (overrides --position)
    #[arg(long)]
    pub y: Option<i32>,

    /// Named position: center, top-left, top-right, bottom-left, bottom-right
    #[arg(long)]
    pub position: Option<String>,

    /// Margin in pixels for named positions (default: 10)
    #[arg(long)]
    pub margin: Option<u32>,

    /// Output quality (1-100, for lossy formats)
    #[arg(long)]
    pub quality: Option<u8>,

    /// Strip metadata from output
    #[arg(long)]
    pub strip: bool,
}

#[cfg(feature = "tiny")]
#[derive(Parser)]
pub struct TinyArgs {
    /// Input image file
    pub input: Option<String>,

    /// Output file path (positional alternative to -o)
    pub output_pos: Option<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Compression quality (1-100, default: PNG n/a, JPEG 75, WebP 75, AVIF 68)
    #[arg(long)]
    pub quality: Option<u8>,

    /// PNG: max number of colors for quantization (2-256, default: 256)
    #[arg(long, default_value = "256")]
    pub max_colors: u16,

    /// PNG: skip quantization, only apply lossless optimization
    #[arg(long)]
    pub lossless: bool,

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

    /// Trim tolerance (for trim): 0-255
    #[arg(long)]
    pub tolerance: Option<i32>,

    /// Saturation factor (for saturate): 0.0=grayscale, 1.0=unchanged, 2.0=double
    #[arg(long)]
    pub factor: Option<f32>,

    /// Sepia intensity (for sepia): 0.0-1.0
    #[arg(long)]
    pub intensity: Option<f32>,

    /// Color levels per channel (for posterize): 2-256
    #[arg(long)]
    pub levels: Option<u8>,
}
