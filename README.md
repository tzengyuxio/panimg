# panimg

[![CI](https://github.com/tzengyuxio/panimg/actions/workflows/ci.yml/badge.svg)](https://github.com/tzengyuxio/panimg/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-2021_edition-orange.svg)](https://www.rust-lang.org/)

[English](README.md) | [日本語](README.ja.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md)

The Swiss Army knife of image processing — built for humans and AI agents alike.

## Features

- **Rich command set** for format conversion, resize, crop, rotate, color adjustment, filters, text, compositing, animation, compression, and more
- **Pipeline engine** — chain multiple operations in a single read/write pass
- **Batch processing** — process multiple files with glob patterns and parallel execution
- **AI-agent friendly** — structured JSON output, `--dry-run`, `--schema`, and `--capabilities` for programmatic use
- **Fast & safe** — built in Rust, SIMD-accelerated resize, zero unsafe code in the core pipeline ([benchmarks](benches/results/REPORT.md))

## Installation

### Homebrew (macOS / Linux)

```bash
brew install tzengyuxio/tap/panimg
```

### Cargo

```bash
cargo install panimg-cli
```

### Build from source

```bash
git clone https://github.com/tzengyuxio/panimg.git
cd panimg
cargo build --release
```

See [Supported Formats](docs/formats.md) for optional codec feature flags (AVIF, JPEG XL, SVG).

## Quick Start

```bash
# Get image info
panimg info photo.jpg --format json

# Convert format
panimg convert photo.png -o photo.webp --quality 80

# Convert color space (requires icc feature)
panimg convert photo.jpg --convert-profile display-p3 -o photo-p3.jpg

# Resize with fit mode
panimg resize photo.jpg --width 800 --height 600 --fit cover -o thumb.jpg

# Color adjustment
panimg saturate photo.jpg --factor 1.5 -o vivid.jpg

# Apply filter
panimg blur photo.jpg --sigma 3.0 -o blurred.jpg

# Add watermark text
panimg text photo.jpg --content "© 2026" --size 24 --color white --position bottom-right -o stamped.jpg

# Smart compression
panimg tiny photo.png -o compressed.png

# Multi-step pipeline
panimg pipeline photo.jpg -o result.jpg --steps "resize --width 800 | blur --sigma 1.5 | grayscale"

# Batch processing
panimg batch convert 'photos/*.png' --output-dir ./webp --to webp --quality 80
```

See [Command Reference](docs/commands.md) for full usage and examples.

## Commands

| Command | Description |
|---------|-------------|
| `info` | Show image metadata and properties |
| `convert` | Convert image between formats |
| `tiny` | Smart compression (like TinyPNG) |
| `resize` | Resize with multiple fit modes |
| `crop` | Crop a rectangular region |
| `rotate` | Rotate by any angle (90/180/270 use fast lossless path) |
| `flip` | Mirror horizontally or vertically |
| `auto-orient` | Auto-rotate based on EXIF orientation |
| `brightness` | Adjust image brightness |
| `contrast` | Adjust image contrast |
| `hue-rotate` | Rotate image hue |
| `saturate` | Adjust color saturation |
| `grayscale` | Convert to grayscale |
| `invert` | Invert (negate) colors |
| `sepia` | Apply sepia tone effect |
| `tint` | Tint with a color |
| `posterize` | Reduce color levels per channel |
| `blur` | Apply Gaussian blur |
| `sharpen` | Sharpen using unsharp mask |
| `edge-detect` | Detect edges (Laplacian kernel) |
| `emboss` | Apply emboss effect |
| `tilt-shift` | Miniature/diorama tilt-shift effect |
| `smart-crop` | Auto-select best crop region (entropy/attention) |
| `draw` | Draw shapes (rect, circle, line) |
| `text` | Draw text with embedded or custom fonts |
| `overlay` | Composite one image on another |
| `trim` | Auto-crop whitespace or similar-colored borders |
| `diff` | Compare two images and visualize differences |
| `frames` | Extract frames from animated GIF |
| `animate` | Assemble images into animated GIF |
| `gif-speed` | Change animation playback speed |
| `pipeline` | Run multiple operations in one pass |
| `batch` | Process multiple files with glob patterns |

See [Command Reference](docs/commands.md) for full usage and examples.

## AI Agent Integration

panimg supports programmatic discovery and structured output for AI agents and automation:

```bash
panimg --capabilities --format json   # Discover all commands and formats
panimg resize --schema                # Get parameter definitions as JSON
panimg resize photo.jpg --width 800 -o out.jpg --dry-run --format json  # Preview without side effects
```

See [AI Integration Guide](docs/ai-integration.md) for exit codes, error format, and integration patterns.

## Documentation

- [Command Reference](docs/commands.md) — full usage and examples for all commands
- [Supported Formats](docs/formats.md) — format table and optional codec feature flags
- [AI Integration Guide](docs/ai-integration.md) — structured output, schema, dry-run, exit codes

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
