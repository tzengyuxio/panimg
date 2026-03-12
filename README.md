# panimg

[![CI](https://github.com/tzengyuxio/panimg/actions/workflows/ci.yml/badge.svg)](https://github.com/tzengyuxio/panimg/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-2021_edition-orange.svg)](https://www.rust-lang.org/)

[English](README.md) | [日本語](README.ja.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md)

A next-generation image processing CLI built in Rust. Modern alternative to ImageMagick with first-class AI agent support.

## Features

- **31 commands** covering format conversion, transforms, filters, color manipulation, animation, compositing, text rendering, and more
- **Structured output**: Human-readable by default, `--format json` for machine consumption
- **Dry-run support**: Preview operations with `--dry-run` before executing
- **Schema introspection**: `--schema` returns parameter definitions as JSON
- **Capabilities discovery**: `--capabilities` lists all supported commands, formats, and features
- **Pipeline engine**: Chain multiple operations in a single read/write pass
- **Batch processing**: Process multiple files with glob patterns and parallel execution
- **Consistent syntax**: All flags use `--key value` format
- **Memory-safe**: Built in Rust with zero unsafe code in the core pipeline
- **Fast**: SIMD-accelerated resize via `fast_image_resize` ([benchmarks](benches/results/REPORT.md))

## Installation

```bash
cargo install panimg-cli
```

Or build from source:

```bash
git clone https://github.com/tzengyuxio/panimg.git
cd panimg
cargo build --release
# Binary at target/release/panimg
```

### Optional codecs

Enable additional format support with feature flags:

```bash
cargo build --release --features avif    # AVIF encode/decode
cargo build --release --features jxl     # JPEG XL decode
cargo build --release --features svg     # SVG rasterization
cargo build --release --features all-codecs  # All of the above
```

## Quick Start

```bash
# Show image metadata
panimg info photo.jpg
panimg info photo.jpg --format json --fields width,height

# Convert formats
panimg convert photo.png -o photo.webp --quality 80

# Resize
panimg resize photo.jpg --width 800 -o thumbnail.jpg

# Crop, rotate, flip
panimg crop photo.jpg --x 100 --y 100 --width 400 --height 300 -o cropped.jpg
panimg rotate photo.jpg --angle 90 -o rotated.jpg
panimg flip photo.jpg --direction horizontal -o flipped.jpg

# Color adjustments
panimg brightness photo.jpg --value 20 -o brighter.jpg
panimg contrast photo.jpg --value 1.5 -o enhanced.jpg
panimg saturate photo.jpg --factor 1.5 -o vivid.jpg
panimg sepia photo.jpg -o vintage.jpg

# Filters
panimg blur photo.jpg --sigma 3.0 -o blurred.jpg
panimg sharpen photo.jpg --sigma 1.5 -o sharp.jpg
panimg edge-detect photo.jpg -o edges.jpg

# Color effects
panimg tint photo.jpg --color red --strength 0.3 -o warm.jpg
panimg posterize photo.jpg --levels 4 -o poster.jpg
panimg grayscale photo.jpg -o bw.jpg

# Drawing
panimg draw photo.jpg --shape rect --x 10 --y 10 --width 100 --height 50 --color red -o annotated.jpg
panimg draw photo.jpg --shape circle --cx 200 --cy 200 --radius 50 --color blue --fill -o marked.jpg

# Text rendering
panimg text photo.jpg --content "Hello World" --size 48 --color white --position center -o titled.jpg
panimg text photo.jpg --content "© 2026" --size 16 --color '#FFFFFF80' --position bottom-right -o stamped.jpg

# Compositing
panimg overlay base.jpg --layer watermark.png --position bottom-right --opacity 0.5 -o watermarked.jpg

# GIF animation
panimg frames animation.gif --output-dir ./frames
panimg animate 'frames/*.png' -o animation.gif --delay 100
panimg gif-speed animation.gif -o fast.gif --speed 2.0

# Image comparison
panimg diff before.png after.png -o diff.png

# Pipeline (multi-step)
panimg pipeline photo.jpg -o result.jpg --steps "resize --width 800 | blur --sigma 1.5 | grayscale"

# Batch processing
panimg batch convert 'photos/*.png' --output-dir ./webp --to webp --quality 80
panimg batch resize 'photos/*.jpg' --output-dir ./thumbs --width 200

# Dry-run & schema
panimg resize photo.jpg --width 800 -o out.jpg --dry-run --format json
panimg resize --schema
panimg --capabilities --format json
```

## Commands

### Basic Operations

| Command | Description |
|---------|-------------|
| `info` | Show image metadata and properties |
| `convert` | Convert image between formats |
| `resize` | Resize an image with multiple fit modes |
| `crop` | Crop a rectangular region |
| `rotate` | Rotate by 90, 180, or 270 degrees |
| `flip` | Mirror horizontally or vertically |
| `auto-orient` | Auto-rotate based on EXIF orientation |

### Color Adjustments

| Command | Description |
|---------|-------------|
| `brightness` | Adjust image brightness |
| `contrast` | Adjust image contrast |
| `hue-rotate` | Rotate image hue |
| `saturate` | Adjust color saturation |
| `grayscale` | Convert to grayscale |
| `invert` | Invert (negate) colors |
| `sepia` | Apply sepia tone effect |
| `tint` | Tint with a color |
| `posterize` | Reduce color levels per channel |

### Filters

| Command | Description |
|---------|-------------|
| `blur` | Apply Gaussian blur |
| `sharpen` | Sharpen using unsharp mask |
| `edge-detect` | Detect edges (Laplacian kernel) |
| `emboss` | Apply emboss effect |

### Drawing & Compositing

| Command | Description |
|---------|-------------|
| `draw` | Draw shapes (rect, circle, line) |
| `text` | Draw text with embedded or custom fonts |
| `overlay` | Composite one image on another |
| `trim` | Auto-crop whitespace or similar-colored borders |
| `diff` | Compare two images and visualize differences |

### Animation

| Command | Description |
|---------|-------------|
| `frames` | Extract individual frames from animated GIF |
| `animate` | Assemble images into animated GIF |
| `gif-speed` | Change animation playback speed |

### Workflow

| Command | Description |
|---------|-------------|
| `pipeline` | Run multiple operations in a single read/write pass |
| `batch` | Process multiple files with glob patterns |

## Supported Formats

| Format | Decode | Encode | Notes |
|--------|--------|--------|-------|
| JPEG | Yes | Yes | Quality 1-100 |
| PNG | Yes | Yes | |
| WebP | Yes | Yes | Quality 1-100 |
| GIF | Yes | Yes | Animated GIF support |
| BMP | Yes | Yes | |
| TIFF | Yes | Yes | |
| QOI | Yes | Yes | |
| AVIF | Yes | Yes | Requires `avif` feature |
| JPEG XL | Yes | No | Requires `jxl` feature, decode only |
| SVG | Yes | No | Requires `svg` feature, rasterization only |

## AI Agent Integration

panimg is designed for seamless integration with AI agents and automation scripts:

```bash
# Discover capabilities programmatically
panimg --capabilities --format json

# Get parameter schemas for any command
panimg resize --schema

# Preview operations without side effects
panimg resize photo.jpg --width 800 -o out.jpg --dry-run --format json

# All output is structured JSON when requested
panimg info photo.jpg --format json --fields width,height,format
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Input file error (not found, permission denied, decode failure) |
| 3 | Output issue (write failure, file exists) |
| 4 | Unsupported format |
| 5 | Bad arguments |

## Error Output

Errors are structured and include actionable suggestions:

```bash
$ panimg convert missing.png out.webp
error: file not found: missing.png
  hint: check that the file path is correct

$ panimg convert missing.png out.webp --format json
{
  "error": "file_not_found",
  "path": "missing.png",
  "suggestion": "check that the file path is correct"
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
