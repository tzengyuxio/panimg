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

LARGE="$IMAGES_DIR/test_4k.png"
if [ ! -f "$LARGE" ]; then
  echo "ERROR: Test image not found: $LARGE. Run setup.sh first."
  exit 1
fi

REPORT="$RESULTS_DIR/memory_report.txt"

echo "==> Measuring peak memory usage (RSS) for resize 4K→800"
echo "  Using /usr/bin/time -l on macOS"
echo ""

{
  echo "# Memory Usage Report — Resize 4K→800"
  echo "# Date: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
  echo "# Image: $LARGE"
  echo ""

  for tool_label in panimg magick vips gm; do
    case "$tool_label" in
      panimg)
        CMD="$PANIMG resize $LARGE --width 800 -o $OUTPUT_DIR/mem_panimg.png"
        ;;
      magick)
        CMD="magick $LARGE -resize 800x $OUTPUT_DIR/mem_magick.png"
        ;;
      vips)
        CMD="vips resize $LARGE $OUTPUT_DIR/mem_vips.png 0.2083"
        ;;
      gm)
        CMD="gm convert $LARGE -resize 800x $OUTPUT_DIR/mem_gm.png"
        ;;
    esac

    echo "--- $tool_label ---"
    # Run 3 times and report each
    for run in 1 2 3; do
      # macOS /usr/bin/time -l outputs to stderr
      TIME_OUTPUT=$(/usr/bin/time -l bash -c "$CMD" 2>&1 1>/dev/null || true)
      # Extract peak RSS (in bytes on macOS)
      RSS_BYTES=$(echo "$TIME_OUTPUT" | grep "maximum resident set size" | awk '{print $1}')
      if [ -n "$RSS_BYTES" ]; then
        RSS_MB=$(echo "scale=1; $RSS_BYTES / 1048576" | bc)
        echo "  run $run: ${RSS_MB} MB (${RSS_BYTES} bytes)"
      else
        echo "  run $run: (could not parse RSS)"
        echo "  raw output: $TIME_OUTPUT"
      fi
    done
    echo ""
  done
} | tee "$REPORT"

echo ""
echo "==> Memory benchmarks complete!"
echo "  Report: $REPORT"
