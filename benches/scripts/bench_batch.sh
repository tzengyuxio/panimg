#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCH_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
KODAK_DIR="$BENCH_DIR/images/kodak"
OUTPUT_DIR="$BENCH_DIR/output"
RESULTS_DIR="$BENCH_DIR/results"

mkdir -p "$OUTPUT_DIR/batch_panimg" "$OUTPUT_DIR/batch_magick" \
         "$OUTPUT_DIR/batch_vips" "$OUTPUT_DIR/batch_gm"

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

if [ ! -d "$KODAK_DIR" ] || [ "$(ls "$KODAK_DIR"/*.png 2>/dev/null | wc -l)" -lt 24 ]; then
  echo "ERROR: Kodak images not found. Run setup.sh first."
  exit 1
fi

WARMUP=3
MIN_RUNS=10
CLEAN_BATCH="find $OUTPUT_DIR/batch_panimg $OUTPUT_DIR/batch_magick $OUTPUT_DIR/batch_vips $OUTPUT_DIR/batch_gm -type f -delete 2>/dev/null; true"

echo "==> Running batch benchmarks (24 Kodak images, resize to 384px width)"
echo ""

# --- Batch resize: panimg batch vs shell loops ---
hyperfine --warmup $WARMUP --min-runs $MIN_RUNS \
  --prepare "$CLEAN_BATCH" \
  --export-json "$RESULTS_DIR/batch_resize.json" \
  --command-name "panimg-batch" \
    "$PANIMG batch resize '$KODAK_DIR/*.png' --width 384 --output-dir $OUTPUT_DIR/batch_panimg --overwrite" \
  --command-name "magick-loop" \
    "for f in $KODAK_DIR/*.png; do magick \"\$f\" -resize 384x \"$OUTPUT_DIR/batch_magick/\$(basename \"\$f\")\"; done" \
  --command-name "vips-loop" \
    "for f in $KODAK_DIR/*.png; do vips resize \"\$f\" \"$OUTPUT_DIR/batch_vips/\$(basename \"\$f\")\" 0.5; done" \
  --command-name "gm-loop" \
    "for f in $KODAK_DIR/*.png; do gm convert \"\$f\" -resize 384x \"$OUTPUT_DIR/batch_gm/\$(basename \"\$f\")\"; done"

echo ""
echo "==> Batch benchmarks complete!"
echo "  Results: $RESULTS_DIR/batch_resize.json"
