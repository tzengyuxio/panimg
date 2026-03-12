#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCH_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
IMAGES_DIR="$BENCH_DIR/images"
OUTPUT_DIR="$BENCH_DIR/output"
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

WARMUP=3
MIN_RUNS=10
SMALL="$IMAGES_DIR/test_768x512.png"
LARGE="$IMAGES_DIR/test_4k.png"
O="$OUTPUT_DIR"

# Verify test images exist
for f in "$SMALL" "$LARGE"; do
  if [ ! -f "$f" ]; then
    echo "ERROR: Test image not found: $f"
    echo "Run setup.sh first."
    exit 1
  fi
done

echo "==> Running single-operation benchmarks"
echo "  panimg: $PANIMG"
echo "  Small:  $SMALL"
echo "  Large:  $LARGE"
echo ""

# Helper: clean output files before each hyperfine run
CLEAN="find $O -maxdepth 1 -type f -name '*.png' -o -name '*.jpg' -o -name '*.webp' | xargs rm -f"

# --- 1. PNG to JPEG conversion ---
echo "--- [1/10] PNG → JPEG (768x512)"
hyperfine --warmup $WARMUP --min-runs $MIN_RUNS \
  --prepare "$CLEAN" \
  --export-json "$RESULTS_DIR/convert_jpeg.json" \
  --command-name "panimg" "$PANIMG convert $SMALL $O/out_panimg.jpg" \
  --command-name "magick" "magick $SMALL $O/out_magick.jpg" \
  --command-name "vips" "vips copy $SMALL $O/out_vips.jpg" \
  --command-name "gm" "gm convert $SMALL $O/out_gm.jpg"

# --- 2. PNG to WebP conversion ---
echo "--- [2/10] PNG → WebP (768x512)"
hyperfine --warmup $WARMUP --min-runs $MIN_RUNS \
  --prepare "$CLEAN" \
  --export-json "$RESULTS_DIR/convert_webp.json" \
  --command-name "panimg" "$PANIMG convert $SMALL $O/out_panimg.webp" \
  --command-name "magick" "magick $SMALL $O/out_magick.webp" \
  --command-name "vips" "vips copy $SMALL $O/out_vips.webp"

# --- 3. Resize 768→384 ---
echo "--- [3/10] Resize 768→384 (small)"
hyperfine --warmup $WARMUP --min-runs $MIN_RUNS \
  --prepare "$CLEAN" \
  --export-json "$RESULTS_DIR/resize_small.json" \
  --command-name "panimg" "$PANIMG resize $SMALL --width 384 -o $O/resized_panimg.png" \
  --command-name "magick" "magick $SMALL -resize 384x $O/resized_magick.png" \
  --command-name "vips" "vips resize $SMALL $O/resized_vips.png 0.5" \
  --command-name "gm" "gm convert $SMALL -resize 384x $O/resized_gm.png"

# --- 4. Resize 4K→800 ---
echo "--- [4/10] Resize 4K→800 (large)"
hyperfine --warmup $WARMUP --min-runs $MIN_RUNS \
  --prepare "$CLEAN" \
  --export-json "$RESULTS_DIR/resize_large.json" \
  --command-name "panimg" "$PANIMG resize $LARGE --width 800 -o $O/resized_4k_panimg.png" \
  --command-name "magick" "magick $LARGE -resize 800x $O/resized_4k_magick.png" \
  --command-name "vips" "vips resize $LARGE $O/resized_4k_vips.png 0.2083" \
  --command-name "gm" "gm convert $LARGE -resize 800x $O/resized_4k_gm.png"

# --- 5. Rotate 90 ---
echo "--- [5/10] Rotate 90° (768x512)"
hyperfine --warmup $WARMUP --min-runs $MIN_RUNS \
  --prepare "$CLEAN" \
  --export-json "$RESULTS_DIR/rotate90.json" \
  --command-name "panimg" "$PANIMG rotate $SMALL --angle 90 -o $O/rot90_panimg.png" \
  --command-name "magick" "magick $SMALL -rotate 90 $O/rot90_magick.png" \
  --command-name "vips" "vips rot $SMALL $O/rot90_vips.png d90" \
  --command-name "gm" "gm convert $SMALL -rotate 90 $O/rot90_gm.png"

# --- 6. Flip horizontal ---
echo "--- [6/10] Flip horizontal (768x512)"
hyperfine --warmup $WARMUP --min-runs $MIN_RUNS \
  --prepare "$CLEAN" \
  --export-json "$RESULTS_DIR/flip.json" \
  --command-name "panimg" "$PANIMG flip $SMALL --direction horizontal -o $O/flip_panimg.png" \
  --command-name "magick" "magick $SMALL -flop $O/flip_magick.png" \
  --command-name "vips" "vips flip $SMALL $O/flip_vips.png horizontal" \
  --command-name "gm" "gm convert $SMALL -flop $O/flip_gm.png"

# --- 7. Blur sigma=2.0 ---
echo "--- [7/10] Blur sigma=2.0 (768x512)"
hyperfine --warmup $WARMUP --min-runs $MIN_RUNS \
  --prepare "$CLEAN" \
  --export-json "$RESULTS_DIR/blur.json" \
  --command-name "panimg" "$PANIMG blur $SMALL --sigma 2.0 -o $O/blur_panimg.png" \
  --command-name "magick" "magick $SMALL -blur 0x2.0 $O/blur_magick.png" \
  --command-name "vips" "vips gaussblur $SMALL $O/blur_vips.png 2.0" \
  --command-name "gm" "gm convert $SMALL -blur 0x2.0 $O/blur_gm.png"

# --- 8. Grayscale ---
echo "--- [8/10] Grayscale (768x512)"
hyperfine --warmup $WARMUP --min-runs $MIN_RUNS \
  --prepare "$CLEAN" \
  --export-json "$RESULTS_DIR/grayscale.json" \
  --command-name "panimg" "$PANIMG grayscale $SMALL -o $O/gray_panimg.png" \
  --command-name "magick" "magick $SMALL -colorspace Gray $O/gray_magick.png" \
  --command-name "vips" "vips colourspace $SMALL $O/gray_vips.png b-w" \
  --command-name "gm" "gm convert $SMALL -colorspace Gray $O/gray_gm.png"

# --- 9. Resize + Grayscale + Blur pipeline (panimg) vs chained commands ---
echo "--- [9/10] Pipeline: resize+grayscale+blur (768x512)"
hyperfine --warmup $WARMUP --min-runs $MIN_RUNS \
  --prepare "$CLEAN" \
  --export-json "$RESULTS_DIR/pipeline.json" \
  --command-name "panimg-pipeline" "$PANIMG pipeline $SMALL -o $O/pipe_panimg.png --steps 'resize --width 384 | grayscale | blur --sigma 2.0'" \
  --command-name "magick" "magick $SMALL -resize 384x -colorspace Gray -blur 0x2.0 $O/pipe_magick.png" \
  --command-name "vips-chain" "vips resize $SMALL $O/_tmp_vp1.png 0.5 && vips colourspace $O/_tmp_vp1.png $O/_tmp_vp2.png b-w && vips gaussblur $O/_tmp_vp2.png $O/pipe_vips.png 2.0"

# --- 10. Resize 4K + Grayscale + Blur pipeline (large) ---
echo "--- [10/10] Pipeline: resize+grayscale+blur (4K)"
hyperfine --warmup $WARMUP --min-runs $MIN_RUNS \
  --prepare "$CLEAN" \
  --export-json "$RESULTS_DIR/pipeline_large.json" \
  --command-name "panimg-pipeline" "$PANIMG pipeline $LARGE -o $O/pipe_4k_panimg.png --steps 'resize --width 800 | grayscale | blur --sigma 2.0'" \
  --command-name "magick" "magick $LARGE -resize 800x -colorspace Gray -blur 0x2.0 $O/pipe_4k_magick.png" \
  --command-name "vips-chain" "vips resize $LARGE $O/_tmp_vp1_4k.png 0.2083 && vips colourspace $O/_tmp_vp1_4k.png $O/_tmp_vp2_4k.png b-w && vips gaussblur $O/_tmp_vp2_4k.png $O/pipe_4k_vips.png 2.0"

echo ""
echo "==> Single-operation benchmarks complete!"
echo "  Results: $RESULTS_DIR/"
