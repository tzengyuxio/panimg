#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCH_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
IMAGES_DIR="$BENCH_DIR/images"
OUTPUT_DIR="$BENCH_DIR/output/quality"
RESULTS_DIR="$BENCH_DIR/results"

mkdir -p "$OUTPUT_DIR" "$RESULTS_DIR"

# Resolve panimg binary
PANIMG=""
if command -v panimg &>/dev/null; then
  PANIMG="panimg"
elif [ -f "$BENCH_DIR/../target/release/panimg" ]; then
  PANIMG="$BENCH_DIR/../target/release/panimg"
else
  echo "ERROR: panimg not found. Run: cargo build --release"
  exit 1
fi

SMALL="$IMAGES_DIR/test_768x512.png"
if [ ! -f "$SMALL" ]; then
  echo "ERROR: Test image not found: $SMALL. Run setup.sh first."
  exit 1
fi

REPORT="$RESULTS_DIR/quality_report.txt"

echo "==> Comparing output quality and file sizes"
echo ""

{
  echo "# Quality Comparison Report"
  echo "# Date: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
  echo "# Source: $SMALL"
  echo ""

  # --- Resize 384px ---
  echo "## Resize to 384px width"
  echo ""
  $PANIMG resize "$SMALL" --width 384 -o "$OUTPUT_DIR/resize_panimg.png"
  magick "$SMALL" -resize 384x "$OUTPUT_DIR/resize_magick.png"
  vips resize "$SMALL" "$OUTPUT_DIR/resize_vips.png" 0.5
  gm convert "$SMALL" -resize 384x "$OUTPUT_DIR/resize_gm.png"

  echo "| Tool | File Size (bytes) | SSIM vs magick |"
  echo "|------|-------------------|----------------|"
  for tool in panimg magick vips gm; do
    FILE="$OUTPUT_DIR/resize_${tool}.png"
    SIZE=$(stat -f%z "$FILE" 2>/dev/null || stat -c%s "$FILE" 2>/dev/null)
    if [ "$tool" = "magick" ]; then
      echo "| $tool | $SIZE | (reference) |"
    else
      SSIM=$(magick compare -metric SSIM "$FILE" "$OUTPUT_DIR/resize_magick.png" null: 2>&1 || true)
      echo "| $tool | $SIZE | $SSIM |"
    fi
  done
  echo ""

  # --- JPEG conversion (quality 85) ---
  echo "## PNG→JPEG (quality 85)"
  echo ""
  $PANIMG convert "$SMALL" "$OUTPUT_DIR/conv_panimg.jpg" --quality 85
  magick "$SMALL" -quality 85 "$OUTPUT_DIR/conv_magick.jpg"
  vips copy "$SMALL" "$OUTPUT_DIR/conv_vips.jpg[Q=85]"
  gm convert "$SMALL" -quality 85 "$OUTPUT_DIR/conv_gm.jpg"

  echo "| Tool | File Size (bytes) | SSIM vs magick |"
  echo "|------|-------------------|----------------|"
  for tool in panimg magick vips gm; do
    FILE="$OUTPUT_DIR/conv_${tool}.jpg"
    SIZE=$(stat -f%z "$FILE" 2>/dev/null || stat -c%s "$FILE" 2>/dev/null)
    if [ "$tool" = "magick" ]; then
      echo "| $tool | $SIZE | (reference) |"
    else
      SSIM=$(magick compare -metric SSIM "$FILE" "$OUTPUT_DIR/conv_magick.jpg" null: 2>&1 || true)
      echo "| $tool | $SIZE | $SSIM |"
    fi
  done
  echo ""

  # --- Blur sigma=2.0 ---
  echo "## Blur sigma=2.0"
  echo ""
  $PANIMG blur "$SMALL" --sigma 2.0 -o "$OUTPUT_DIR/blur_panimg.png"
  magick "$SMALL" -blur 0x2.0 "$OUTPUT_DIR/blur_magick.png"
  vips gaussblur "$SMALL" "$OUTPUT_DIR/blur_vips.png" 2.0
  gm convert "$SMALL" -blur 0x2.0 "$OUTPUT_DIR/blur_gm.png"

  echo "| Tool | File Size (bytes) | SSIM vs magick |"
  echo "|------|-------------------|----------------|"
  for tool in panimg magick vips gm; do
    FILE="$OUTPUT_DIR/blur_${tool}.png"
    SIZE=$(stat -f%z "$FILE" 2>/dev/null || stat -c%s "$FILE" 2>/dev/null)
    if [ "$tool" = "magick" ]; then
      echo "| $tool | $SIZE | (reference) |"
    else
      SSIM=$(magick compare -metric SSIM "$FILE" "$OUTPUT_DIR/blur_magick.png" null: 2>&1 || true)
      echo "| $tool | $SIZE | $SSIM |"
    fi
  done

} | tee "$REPORT"

echo ""
echo "==> Quality comparison complete!"
echo "  Report: $REPORT"
