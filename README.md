# panimg

A next-generation image processing CLI built in Rust. Modern alternative to ImageMagick with first-class AI agent support.

## Features

- **Structured output**: Human-readable by default, `--format json` for machine consumption
- **Dry-run support**: Preview operations with `--dry-run` before executing
- **Schema introspection**: `--schema` returns parameter definitions as JSON
- **Capabilities discovery**: `--capabilities` lists all supported commands, formats, and features
- **Consistent syntax**: All flags use `--key value` format
- **Memory-safe**: Built in Rust with zero unsafe code in the core pipeline
- **Fast**: SIMD-accelerated resize via `fast_image_resize`

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

# Get metadata as JSON
panimg info photo.jpg --format json

# Get specific fields only
panimg info photo.jpg --format json --fields width,height

# Convert formats
panimg convert photo.png -o photo.webp --quality 80

# Resize an image
panimg resize photo.jpg --width 800 -o thumbnail.jpg

# Preview what would happen (dry-run)
panimg resize photo.jpg --width 800 -o thumbnail.jpg --dry-run --format json

# Show parameter schema for a command
panimg resize --schema

# List all capabilities
panimg --capabilities --format json
```

## Supported Formats

| Format | Decode | Encode | Notes |
|--------|--------|--------|-------|
| JPEG | Yes | Yes | Quality 1-100 |
| PNG | Yes | Yes | |
| WebP | Yes | Yes | Quality 1-100 |
| GIF | Yes | Yes | |
| BMP | Yes | Yes | |
| TIFF | Yes | Yes | |
| QOI | Yes | Yes | |
| AVIF | Yes | Yes | Requires `avif` feature |
| JPEG XL | Yes | No | Requires `jxl` feature, decode only |
| SVG | Yes | No | Requires `svg` feature, rasterization only |

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
