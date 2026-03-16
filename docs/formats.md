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
| PDF | Yes | No | `pdf` | Multi-page support; `--dpi`, `--page`, `pdf-pages` command |
| HEIC | Yes | No | `heic` | macOS only; requires system libheif ≥ 1.21 |
| PSD | Yes | No | `psd` | Photoshop format; merged composite + layer extraction |

### Building with Optional Codecs

Enable individual codecs:

```bash
cargo build --release --features avif        # AVIF encode/decode
cargo build --release --features jxl         # JPEG XL decode
cargo build --release --features svg         # SVG rasterization
cargo build --release --features pdf         # PDF rasterization
cargo build --release --features heic        # HEIC decode (macOS only)
cargo build --release --features psd         # PSD decode + layer extraction
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

Supported extensions: `.jpg` / `.jpeg`, `.png`, `.webp`, `.gif`, `.bmp`, `.tiff` / `.tif`, `.qoi`, `.avif`, `.jxl`, `.svg` (input only), `.pdf` (input only), `.heic` / `.heif` (input only, macOS), `.psd` (input only).
