# Supported Formats

## Built-in Formats

These formats are supported out of the box with no additional feature flags:

| Format | Decode | Encode | Notes |
|--------|--------|--------|-------|
| JPEG | Yes | Yes | Quality 1-100 |
| PNG | Yes | Yes | |
| WebP | Yes | Yes | Quality 1-100 |
| GIF | Yes | Yes | Animated GIF support |
| BMP | Yes | Yes | |
| TIFF | Yes | Yes | |
| QOI | Yes | Yes | |

## Optional Codec Feature Flags

Additional formats can be enabled at build time using Cargo feature flags:

| Format | Decode | Encode | Feature Flag | Notes |
|--------|--------|--------|-------------|-------|
| AVIF | Yes | Yes | `avif` | AVIF encode/decode |
| JPEG XL | Yes | No | `jxl` | Decode only |
| SVG | Yes | No | `svg` | Rasterization only |

### Building with Optional Codecs

Enable individual codecs:

```bash
cargo build --release --features avif        # AVIF encode/decode
cargo build --release --features jxl         # JPEG XL decode
cargo build --release --features svg         # SVG rasterization
```

Enable all optional codecs at once:

```bash
cargo build --release --features all-codecs
```

When installing via Cargo:

```bash
cargo install panimg-cli --features avif
cargo install panimg-cli --features all-codecs
```

## Format Detection

panimg determines the output format from the file extension specified in `-o` / `--output`. For `batch convert`, use the `--to` flag to specify the target format.

Supported extensions: `.jpg` / `.jpeg`, `.png`, `.webp`, `.gif`, `.bmp`, `.tiff` / `.tif`, `.qoi`, `.avif`, `.jxl`, `.svg` (input only).
