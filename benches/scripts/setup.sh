#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCH_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
IMAGES_DIR="$BENCH_DIR/images"
KODAK_DIR="$IMAGES_DIR/kodak"

# --- Check required tools ---
echo "==> Checking required tools..."
MISSING=()
for cmd in magick vips gm hyperfine; do
  if ! command -v "$cmd" &>/dev/null; then
    MISSING+=("$cmd")
  fi
done

# Check panimg: try PATH first, then cargo target
PANIMG=""
if command -v panimg &>/dev/null; then
  PANIMG="$(command -v panimg)"
elif [ -f "$BENCH_DIR/../target/release/panimg" ]; then
  PANIMG="$BENCH_DIR/../target/release/panimg"
else
  MISSING+=("panimg (run: cargo build --release)")
fi

if [ ${#MISSING[@]} -gt 0 ]; then
  echo "ERROR: Missing tools: ${MISSING[*]}"
  echo "Install with: brew install hyperfine vips graphicsmagick imagemagick"
  exit 1
fi
echo "  All tools found."

# --- Download Kodak test images ---
echo "==> Downloading Kodak test images..."
mkdir -p "$KODAK_DIR"

KODAK_BASE="http://r0k.us/graphics/kodak/kodak"
for i in $(seq -w 1 24); do
  FILE="$KODAK_DIR/kodim${i}.png"
  if [ -f "$FILE" ]; then
    continue
  fi
  echo "  Downloading kodim${i}.png..."
  curl -sL "${KODAK_BASE}/kodim${i}.png" -o "$FILE"
done
echo "  Kodak images ready (24 files)."

# --- Generate synthetic test images ---
echo "==> Generating synthetic test images..."

# 768x512 (small, for quick tests)
if [ ! -f "$IMAGES_DIR/test_768x512.png" ]; then
  magick -size 768x512 plasma:fractal "$IMAGES_DIR/test_768x512.png"
  echo "  Generated test_768x512.png"
fi

# 3840x2160 (4K)
if [ ! -f "$IMAGES_DIR/test_4k.png" ]; then
  magick -size 3840x2160 plasma:fractal "$IMAGES_DIR/test_4k.png"
  echo "  Generated test_4k.png (3840x2160)"
fi

# 7680x4320 (8K)
if [ ! -f "$IMAGES_DIR/test_8k.png" ]; then
  magick -size 7680x4320 plasma:fractal "$IMAGES_DIR/test_8k.png"
  echo "  Generated test_8k.png (7680x4320)"
fi

echo ""
echo "==> Setup complete!"
echo "  Images dir: $IMAGES_DIR"
echo "  Kodak:      $KODAK_DIR (24 PNGs, 768x512)"
echo "  Synthetic:  test_768x512.png, test_4k.png, test_8k.png"
