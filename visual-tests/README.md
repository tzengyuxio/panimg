# panimg Visual Test Suite

Batch-run all panimg image processing commands and generate an annotated HTML gallery for visual (human-eye) verification.

## Prerequisites

- Rust toolchain (for building panimg)
- [ImageMagick](https://imagemagick.org/) (`magick` command)

## Usage

```bash
bash visual-tests/run.sh
```

This will:

1. Build `panimg` in release mode with the `text` feature
2. Download test images if not already present
3. Run all 45 test cases covering every panimg command
4. Generate annotated comparison images (source vs result + command info)
5. Produce an HTML gallery at `visual-tests/results/gallery.html`

## Viewing Results

```bash
open visual-tests/results/gallery.html
```

Each card in the gallery shows:
- **Source** image (left) and **Result** image (right)
- The exact `panimg` command used
- The command's stdout output

## Test Coverage

| Category | Commands |
|----------|----------|
| Transform | resize, crop, rotate, flip, auto-orient |
| Convert | convert (PNG→JPEG) |
| Adjust | brightness, contrast, hue-rotate, saturate |
| Color | grayscale, invert, sepia, tint, posterize |
| Filter | blur, sharpen, edge-detect, emboss |
| Draw | draw (rect, circle, line), text |
| Composite | overlay, trim, diff |
| Pipeline | pipeline (multi-step) |
| Animation | animate, frames, gif-speed |

## Notes

- Results are written to `visual-tests/results/` (gitignored)
- Temporary files are cleaned up automatically after each run
