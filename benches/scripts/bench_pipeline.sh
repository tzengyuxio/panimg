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

for f in "$SMALL" "$LARGE"; do
  if [ ! -f "$f" ]; then
    echo "ERROR: Test image not found: $f. Run setup.sh first."
    exit 1
  fi
done

CLEAN="find $O -maxdepth 1 -type f \( -name 'pipe_*' -o -name '_pipe_*' -o -name '_vp_*' -o -name '_tmp_*' \) -delete"

echo "==> Running pipeline benchmarks"
echo "  Comparing panimg single-pass pipeline vs multi-tool chaining"
echo ""

# --- Pipeline: resize + grayscale + blur (small) ---
echo "--- Pipeline: resize(384)+grayscale+blur(2.0) on 768x512"
hyperfine --warmup $WARMUP --min-runs $MIN_RUNS \
  --prepare "$CLEAN" \
  --export-json "$RESULTS_DIR/pipeline_vs_chain_small.json" \
  --command-name "panimg-pipeline" \
    "$PANIMG pipeline $SMALL -o $O/pipe_s_panimg.png --steps 'resize --width 384 | grayscale | blur --sigma 2.0'" \
  --command-name "panimg-sequential" \
    "$PANIMG resize $SMALL --width 384 -o $O/_pipe_s1.png && $PANIMG grayscale $O/_pipe_s1.png -o $O/_pipe_s2.png && $PANIMG blur $O/_pipe_s2.png --sigma 2.0 -o $O/pipe_s_seq.png" \
  --command-name "magick-single" \
    "magick $SMALL -resize 384x -colorspace Gray -blur 0x2.0 $O/pipe_s_magick.png" \
  --command-name "vips-chain" \
    "vips resize $SMALL $O/_vp_s1.png 0.5 && vips colourspace $O/_vp_s1.png $O/_vp_s2.png b-w && vips gaussblur $O/_vp_s2.png $O/pipe_s_vips.png 2.0"

# --- Pipeline: resize + grayscale + blur (4K) ---
echo "--- Pipeline: resize(800)+grayscale+blur(2.0) on 4K"
hyperfine --warmup $WARMUP --min-runs $MIN_RUNS \
  --prepare "$CLEAN" \
  --export-json "$RESULTS_DIR/pipeline_vs_chain_large.json" \
  --command-name "panimg-pipeline" \
    "$PANIMG pipeline $LARGE -o $O/pipe_l_panimg.png --steps 'resize --width 800 | grayscale | blur --sigma 2.0'" \
  --command-name "panimg-sequential" \
    "$PANIMG resize $LARGE --width 800 -o $O/_pipe_l1.png && $PANIMG grayscale $O/_pipe_l1.png -o $O/_pipe_l2.png && $PANIMG blur $O/_pipe_l2.png --sigma 2.0 -o $O/pipe_l_seq.png" \
  --command-name "magick-single" \
    "magick $LARGE -resize 800x -colorspace Gray -blur 0x2.0 $O/pipe_l_magick.png" \
  --command-name "vips-chain" \
    "vips resize $LARGE $O/_vp_l1.png 0.2083 && vips colourspace $O/_vp_l1.png $O/_vp_l2.png b-w && vips gaussblur $O/_vp_l2.png $O/pipe_l_vips.png 2.0"

echo ""
echo "==> Pipeline benchmarks complete!"
echo "  Results: $RESULTS_DIR/pipeline_vs_chain_*.json"
