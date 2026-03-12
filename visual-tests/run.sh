#!/usr/bin/env bash
#
# Visual Test Suite for panimg
# Runs all image processing commands and generates annotated result images + HTML gallery.
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RESULTS_DIR="$SCRIPT_DIR/results"
TMP_DIR="$SCRIPT_DIR/tmp"
SOURCE="$PROJECT_ROOT/benches/images/kodak/kodim01.png"
PANIMG="$PROJECT_ROOT/target/release/panimg"

# Counters
PASS=0
FAIL=0
TOTAL=0
TEST_NUM=0

# Colors for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Font for ImageMagick annotations (macOS system font)
FONT="/System/Library/Fonts/Geneva.ttf"
if [[ ! -f "$FONT" ]]; then
    FONT="/System/Library/Fonts/Supplemental/Arial.ttf"
fi
if [[ ! -f "$FONT" ]]; then
    FONT=""  # Fall back to ImageMagick default
fi
FONT_ARG=()
if [[ -n "$FONT" ]]; then
    FONT_ARG=(-font "$FONT")
fi

# ---------- Prerequisite checks ----------

echo -e "${CYAN}=== panimg Visual Test Suite ===${NC}"
echo ""

# Build panimg
echo -e "${YELLOW}Building panimg (release, with text feature)...${NC}"
(cd "$PROJECT_ROOT" && cargo build --release --features text 2>&1)
echo -e "${GREEN}Build complete.${NC}"

if [[ ! -x "$PANIMG" ]]; then
    echo -e "${RED}ERROR: panimg binary not found at $PANIMG${NC}"
    exit 1
fi

if ! command -v magick &>/dev/null; then
    echo -e "${RED}ERROR: ImageMagick (magick) not found. See https://imagemagick.org/script/download.php${NC}"
    exit 1
fi

if [[ ! -f "$SOURCE" ]]; then
    echo -e "${YELLOW}Source image not found. Downloading...${NC}"
    mkdir -p "$(dirname "$SOURCE")"
    curl -sL "http://r0k.us/graphics/kodak/kodak/kodim01.png" -o "$SOURCE"
fi

if [[ ! -f "$SOURCE" ]]; then
    echo -e "${RED}ERROR: Source image still not found at $SOURCE${NC}"
    exit 1
fi

# ---------- Prepare directories ----------

rm -rf "$RESULTS_DIR" "$TMP_DIR"
mkdir -p "$RESULTS_DIR" "$TMP_DIR"

# ---------- Helper: run_test ----------

run_test() {
    local name="$1"
    local description="$2"
    shift 2
    local cmd_args=("$@")

    TEST_NUM=$((TEST_NUM + 1))
    TOTAL=$((TOTAL + 1))
    local num=$(printf "%02d" "$TEST_NUM")
    local out_file="$RESULTS_DIR/${num}_${name}.png"

    echo -ne "  [${num}] ${description}... "

    # Build the full command string for display
    local display_cmd="panimg ${cmd_args[*]}"

    # Execute panimg command, capture stdout and exit code
    local stdout=""
    local exit_code=0
    stdout=$("$PANIMG" "${cmd_args[@]}" 2>&1) || exit_code=$?

    if [[ $exit_code -ne 0 ]]; then
        echo -e "${RED}FAIL${NC} (exit code $exit_code)"
        FAIL=$((FAIL + 1))
        # Create a failure placeholder image
        magick -size 600x200 xc:'#1a1a2e' \
            "${FONT_ARG[@]}" -fill '#ff4444' -pointsize 24 \
            -gravity Center -annotate +0-30 "[${num}] ${description}" \
            -fill '#ff8888' -pointsize 16 \
            -annotate +0+20 "FAILED: exit code ${exit_code}" \
            -fill '#aaaaaa' -pointsize 14 \
            -annotate +0+50 "$ ${display_cmd}" \
            "$out_file"
        return
    fi

    echo -e "${GREEN}OK${NC}"
    PASS=$((PASS + 1))

    # Determine the actual result file path
    # The output file is determined by the -o flag in cmd_args
    local result_file=""
    for i in "${!cmd_args[@]}"; do
        if [[ "${cmd_args[$i]}" == "-o" ]] && [[ $((i + 1)) -lt ${#cmd_args[@]} ]]; then
            result_file="${cmd_args[$((i + 1))]}"
            break
        fi
        if [[ "${cmd_args[$i]}" == "--output-dir" ]] && [[ $((i + 1)) -lt ${#cmd_args[@]} ]]; then
            # For frames command, pick the first extracted frame
            local frames_dir="${cmd_args[$((i + 1))]}"
            result_file=$(ls "$frames_dir"/*.png 2>/dev/null | head -1 || true)
            break
        fi
    done

    if [[ -z "$result_file" ]] || [[ ! -f "$result_file" ]]; then
        # No result file found, create info-only annotation
        magick -size 600x150 xc:'#1a1a2e' \
            "${FONT_ARG[@]}" -fill white -pointsize 20 \
            -gravity NorthWest -annotate +10+20 "[${num}] ${description}" \
            -fill '#aaaaaa' -pointsize 14 \
            -annotate +10+60 "$ ${display_cmd}" \
            -fill '#88ccff' -pointsize 14 \
            -annotate +10+90 "→ ${stdout}" \
            "$out_file"
        return
    fi

    # Create annotated comparison image:
    # Top: source (left) + result (right), resized to 300px height
    # Bottom: info text on dark background

    local src_resized="$TMP_DIR/src_${num}.png"
    local res_resized="$TMP_DIR/res_${num}.png"

    # Resize source and result to 300px height for comparison
    magick "$SOURCE" -resize x300 "$src_resized"

    # Handle GIF results: take first frame for static comparison
    if [[ "$result_file" == *.gif ]]; then
        magick "${result_file}[0]" -resize x300 "$res_resized"
        # Also copy the GIF to results for gallery playback
        cp "$result_file" "$RESULTS_DIR/${num}_${name}.gif"
    else
        magick "$result_file" -resize x300 "$res_resized"
    fi

    # Get dimensions for layout
    local src_w res_w
    src_w=$(magick identify -format "%w" "$src_resized")
    res_w=$(magick identify -format "%w" "$res_resized")
    local total_w=$(( src_w + res_w + 30 ))  # 30px gap
    [[ $total_w -lt 600 ]] && total_w=600

    # Truncate stdout for display (first line, max 80 chars)
    local display_stdout="${stdout%%$'\n'*}"
    if [[ ${#display_stdout} -gt 100 ]]; then
        display_stdout="${display_stdout:0:97}..."
    fi

    # Create the comparison strip (source | result)
    local comparison="$TMP_DIR/cmp_${num}.png"
    magick "$src_resized" "$res_resized" +append -gravity Center \
        -background '#0d0d0d' -splice 10x0+${src_w}+0 \
        "$comparison"

    # Get comparison width for consistent info bar
    local cmp_w
    cmp_w=$(magick identify -format "%w" "$comparison")
    [[ $cmp_w -lt 600 ]] && cmp_w=600

    # Create info bar
    local info_bar="$TMP_DIR/info_${num}.png"
    magick -size "${cmp_w}x100" xc:'#1a1a2e' \
        "${FONT_ARG[@]}" -fill white -pointsize 18 \
        -gravity NorthWest -annotate +10+10 "[${num}] ${description}" \
        -fill '#aaaaaa' -pointsize 14 \
        -annotate +10+40 "$ ${display_cmd}" \
        -fill '#88ccff' -pointsize 13 \
        -annotate +10+65 "→ ${display_stdout}" \
        "$info_bar"

    # Add "Source" / "Result" labels on comparison
    local labeled_cmp="$TMP_DIR/lcmp_${num}.png"
    local label_x_result=$((src_w + 15))
    magick "$comparison" \
        "${FONT_ARG[@]}" -fill '#ffffff80' -pointsize 14 \
        -gravity NorthWest \
        -annotate +5+5 "Source" \
        -annotate +${label_x_result}+5 "Result" \
        "$labeled_cmp"

    # Stack comparison + info bar vertically
    magick "$labeled_cmp" "$info_bar" -append "$out_file"
}

# Like run_test but treats exit code 1 as success (for diff command)
run_test_allow_exit1() {
    local name="$1"
    local description="$2"
    shift 2
    local cmd_args=("$@")

    TEST_NUM=$((TEST_NUM + 1))
    TOTAL=$((TOTAL + 1))
    local num=$(printf "%02d" "$TEST_NUM")
    local out_file="$RESULTS_DIR/${num}_${name}.png"

    echo -ne "  [${num}] ${description}... "

    local display_cmd="panimg ${cmd_args[*]}"
    local stdout=""
    local exit_code=0
    stdout=$("$PANIMG" "${cmd_args[@]}" 2>&1) || exit_code=$?

    if [[ $exit_code -ne 0 && $exit_code -ne 1 ]]; then
        echo -e "${RED}FAIL${NC} (exit code $exit_code)"
        FAIL=$((FAIL + 1))
        magick -size 600x200 xc:'#1a1a2e' \
            "${FONT_ARG[@]}" -fill '#ff4444' -pointsize 24 \
            -gravity Center -annotate +0-30 "[${num}] ${description}" \
            -fill '#ff8888' -pointsize 16 \
            -annotate +0+20 "FAILED: exit code ${exit_code}" \
            -fill '#aaaaaa' -pointsize 14 \
            -annotate +0+50 "$ ${display_cmd}" \
            "$out_file"
        return
    fi

    echo -e "${GREEN}OK${NC}"
    PASS=$((PASS + 1))

    # Find result file from -o flag
    local result_file=""
    for i in "${!cmd_args[@]}"; do
        if [[ "${cmd_args[$i]}" == "-o" ]] && [[ $((i + 1)) -lt ${#cmd_args[@]} ]]; then
            result_file="${cmd_args[$((i + 1))]}"
            break
        fi
    done

    if [[ -z "$result_file" ]] || [[ ! -f "$result_file" ]]; then
        return
    fi

    local src_resized="$TMP_DIR/src_${num}.png"
    local res_resized="$TMP_DIR/res_${num}.png"
    magick "$SOURCE" -resize x300 "$src_resized"
    magick "$result_file" -resize x300 "$res_resized"

    local src_w res_w
    src_w=$(magick identify -format "%w" "$src_resized")
    res_w=$(magick identify -format "%w" "$res_resized")

    local display_stdout="${stdout%%$'\n'*}"
    if [[ ${#display_stdout} -gt 100 ]]; then
        display_stdout="${display_stdout:0:97}..."
    fi

    local comparison="$TMP_DIR/cmp_${num}.png"
    magick "$src_resized" "$res_resized" +append -gravity Center \
        -background '#0d0d0d' -splice 10x0+${src_w}+0 \
        "$comparison"

    local cmp_w
    cmp_w=$(magick identify -format "%w" "$comparison")
    [[ $cmp_w -lt 600 ]] && cmp_w=600

    local info_bar="$TMP_DIR/info_${num}.png"
    magick -size "${cmp_w}x100" xc:'#1a1a2e' \
        "${FONT_ARG[@]}" -fill white -pointsize 18 \
        -gravity NorthWest -annotate +10+10 "[${num}] ${description}" \
        -fill '#aaaaaa' -pointsize 14 \
        -annotate +10+40 "$ ${display_cmd}" \
        -fill '#88ccff' -pointsize 13 \
        -annotate +10+65 "→ ${display_stdout}" \
        "$info_bar"

    local labeled_cmp="$TMP_DIR/lcmp_${num}.png"
    local label_x_result=$((src_w + 15))
    magick "$comparison" \
        "${FONT_ARG[@]}" -fill '#ffffff80' -pointsize 14 \
        -gravity NorthWest \
        -annotate +5+5 "Source" \
        -annotate +${label_x_result}+5 "Result" \
        "$labeled_cmp"

    magick "$labeled_cmp" "$info_bar" -append "$out_file"
}

# ---------- Generate auxiliary assets ----------

echo ""
echo -e "${YELLOW}Generating auxiliary test assets...${NC}"

# Overlay: semi-transparent red circle
magick -size 200x200 xc:none \
    -fill 'rgba(255,0,0,180)' -draw 'circle 100,100 100,20' \
    "$TMP_DIR/overlay.png"

# Padded image for trim test
magick "$SOURCE" -bordercolor white -border 50x50 "$TMP_DIR/padded.png"

# Blurred image for diff test
"$PANIMG" blur "$SOURCE" -o "$TMP_DIR/blurred.png" --sigma 5

# Multi-frame images for animate/frames/gif-speed
"$PANIMG" hue-rotate "$SOURCE" -o "$TMP_DIR/frame_01.png" --degrees 0
"$PANIMG" hue-rotate "$SOURCE" -o "$TMP_DIR/frame_02.png" --degrees 90
"$PANIMG" hue-rotate "$SOURCE" -o "$TMP_DIR/frame_03.png" --degrees 180
"$PANIMG" hue-rotate "$SOURCE" -o "$TMP_DIR/frame_04.png" --degrees 270

# Pre-generate animated GIF for frames/gif-speed tests
"$PANIMG" animate "$TMP_DIR/frame_*.png" -o "$TMP_DIR/animated.gif" --delay 200

echo -e "${GREEN}Assets ready.${NC}"
echo ""

# ---------- Shorthand variables ----------

S="$SOURCE"
OVERLAY="$TMP_DIR/overlay.png"
PADDED="$TMP_DIR/padded.png"
BLURRED="$TMP_DIR/blurred.png"
GIF="$TMP_DIR/animated.gif"

# ---------- Run all tests ----------

echo -e "${CYAN}Running visual tests...${NC}"
echo ""

# -- Resize --
run_test "resize_width" "Resize: width 384" \
    resize "$S" -o "$TMP_DIR/t01.png" --width 384
run_test "resize_cover" "Resize: fit cover 300×300" \
    resize "$S" -o "$TMP_DIR/t02.png" --width 300 --height 300 --fit cover
run_test "resize_contain" "Resize: fit contain 300×300" \
    resize "$S" -o "$TMP_DIR/t03.png" --width 300 --height 300 --fit contain

# -- Crop --
run_test "crop_region" "Crop: 400×300 at (100,50)" \
    crop "$S" -o "$TMP_DIR/t04.png" --x 100 --y 50 --width 400 --height 300
run_test "crop_topleft" "Crop: top-left 256×256" \
    crop "$S" -o "$TMP_DIR/t05.png" --x 0 --y 0 --width 256 --height 256

# -- Transform --
run_test "rotate_90" "Rotate: 90°" \
    rotate "$S" -o "$TMP_DIR/t06.png" --angle 90
run_test "rotate_180" "Rotate: 180°" \
    rotate "$S" -o "$TMP_DIR/t07.png" --angle 180
run_test "flip_horizontal" "Flip: horizontal" \
    flip "$S" -o "$TMP_DIR/t08.png" --direction horizontal
run_test "flip_vertical" "Flip: vertical" \
    flip "$S" -o "$TMP_DIR/t09.png" --direction vertical
run_test "auto_orient" "Auto-orient (no-op on test image)" \
    auto-orient "$S" -o "$TMP_DIR/t10.png"

# -- Convert --
run_test "convert_jpeg" "Convert: PNG → JPEG q80" \
    convert "$S" -o "$TMP_DIR/t11.jpg" --quality 80

# -- Brightness / Contrast --
run_test "brightness_up" "Brightness: +40" \
    brightness "$S" -o "$TMP_DIR/t12.png" --value 40
run_test "brightness_down" "Brightness: −40" \
    brightness "$S" -o "$TMP_DIR/t13.png" --value=-40
run_test "contrast_up" "Contrast: +50" \
    contrast "$S" -o "$TMP_DIR/t14.png" --value 50
run_test "contrast_down" "Contrast: −50" \
    contrast "$S" -o "$TMP_DIR/t15.png" --value=-50

# -- Color --
run_test "hue_rotate_90" "Hue-rotate: 90°" \
    hue-rotate "$S" -o "$TMP_DIR/t16.png" --degrees 90
run_test "hue_rotate_180" "Hue-rotate: 180°" \
    hue-rotate "$S" -o "$TMP_DIR/t17.png" --degrees 180
run_test "saturate_high" "Saturate: ×2.0" \
    saturate "$S" -o "$TMP_DIR/t18.png" --factor 2.0
run_test "saturate_low" "Saturate: ×0.3" \
    saturate "$S" -o "$TMP_DIR/t19.png" --factor 0.3
run_test "grayscale" "Grayscale" \
    grayscale "$S" -o "$TMP_DIR/t20.png"
run_test "invert" "Invert colors" \
    invert "$S" -o "$TMP_DIR/t21.png"
run_test "sepia_full" "Sepia: intensity 1.0" \
    sepia "$S" -o "$TMP_DIR/t22.png" --intensity 1.0
run_test "sepia_half" "Sepia: intensity 0.5" \
    sepia "$S" -o "$TMP_DIR/t23.png" --intensity 0.5
run_test "tint_red" "Tint: red 0.5" \
    tint "$S" -o "$TMP_DIR/t24.png" --color red --strength 0.5
run_test "tint_blue" "Tint: blue 0.3" \
    tint "$S" -o "$TMP_DIR/t25.png" --color blue --strength 0.3
run_test "posterize_4" "Posterize: 4 levels" \
    posterize "$S" -o "$TMP_DIR/t26.png" --levels 4
run_test "posterize_2" "Posterize: 2 levels" \
    posterize "$S" -o "$TMP_DIR/t27.png" --levels 2

# -- Filters --
run_test "blur_light" "Blur: sigma 2.0" \
    blur "$S" -o "$TMP_DIR/t28.png" --sigma 2.0
run_test "blur_heavy" "Blur: sigma 8.0" \
    blur "$S" -o "$TMP_DIR/t29.png" --sigma 8.0
run_test "sharpen" "Sharpen: sigma 2.0" \
    sharpen "$S" -o "$TMP_DIR/t30.png" --sigma 2.0
run_test "edge_detect" "Edge detect" \
    edge-detect "$S" -o "$TMP_DIR/t31.png"
run_test "emboss" "Emboss" \
    emboss "$S" -o "$TMP_DIR/t32.png"

# -- Draw --
run_test "draw_rect" "Draw: filled rect" \
    draw "$S" -o "$TMP_DIR/t33.png" --shape rect --x 50 --y 50 --width 200 --height 100 --color red --fill
run_test "draw_circle" "Draw: circle stroke" \
    draw "$S" -o "$TMP_DIR/t34.png" --shape circle --cx 384 --cy 256 --radius 100 --color '#00FF00' --thickness 4
run_test "draw_line" "Draw: line" \
    draw "$S" -o "$TMP_DIR/t35.png" --shape line --x1 0 --y1 0 --x2 768 --y2 512 --color yellow --thickness 3

# -- Text --
run_test "text_center" "Text: center 64px" \
    text "$S" -o "$TMP_DIR/t36.png" --content "panimg" --size 64 --color white --position center
run_test "text_watermark" "Text: bottom-right watermark" \
    text "$S" -o "$TMP_DIR/t37.png" --content "Watermark" --size 24 --color white --position bottom-right

# -- Overlay --
run_test "overlay_center" "Overlay: center 50% opacity" \
    overlay "$S" -o "$TMP_DIR/t38.png" --layer "$OVERLAY" --position center --opacity 0.5

# -- Trim --
run_test "trim" "Trim: tolerance 10" \
    trim "$PADDED" -o "$TMP_DIR/t39.png" --tolerance 10

# -- Diff --
# diff returns exit code 1 when images differ (expected), so handle it specially
run_test_allow_exit1 "diff" "Diff: vs blurred" \
    diff "$S" "$BLURRED" -o "$TMP_DIR/t40.png"

# -- Pipeline --
run_test "pipeline_resize_blur_gray" "Pipeline: resize+blur+grayscale" \
    pipeline "$S" -o "$TMP_DIR/t41.png" --steps 'resize --width 400 | blur --sigma 2 | grayscale'
run_test "pipeline_sepia_contrast_posterize" "Pipeline: sepia+contrast+posterize" \
    pipeline "$S" -o "$TMP_DIR/t42.png" --steps 'sepia --intensity 0.8 | contrast --value 30 | posterize --levels 6'

# -- Animation --
run_test "animate" "Animate: 4 frames → GIF" \
    animate "$TMP_DIR/frame_*.png" -o "$TMP_DIR/t43.gif" --delay 200

FRAMES_DIR="$TMP_DIR/extracted_frames"
mkdir -p "$FRAMES_DIR"
run_test "frames" "Frames: GIF → frames" \
    frames "$GIF" --output-dir "$FRAMES_DIR"

run_test "gif_speed" "GIF speed: 2× faster" \
    gif-speed "$GIF" -o "$TMP_DIR/t45.gif" --speed 2.0

echo ""

# ---------- Generate gallery HTML ----------

echo -e "${YELLOW}Generating gallery HTML...${NC}"

TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')

cat > "$RESULTS_DIR/gallery.html" << 'GALLERY_HEAD'
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>panimg Visual Test Gallery</title>
<style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body {
    background: #0d0d0d;
    color: #e0e0e0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, monospace;
    padding: 20px;
  }
  header {
    text-align: center;
    padding: 30px 0;
    border-bottom: 1px solid #333;
    margin-bottom: 30px;
  }
  header h1 { color: #88ccff; font-size: 28px; margin-bottom: 10px; }
  header .meta { color: #888; font-size: 14px; }
  header .summary { margin-top: 10px; font-size: 16px; }
  .pass { color: #44cc44; }
  .fail { color: #ff4444; }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(500px, 1fr));
    gap: 20px;
    max-width: 1600px;
    margin: 0 auto;
  }
  .card {
    background: #1a1a2e;
    border-radius: 8px;
    overflow: hidden;
    border: 1px solid #333;
    transition: border-color 0.2s;
  }
  .card:hover { border-color: #88ccff; }
  .card img {
    width: 100%;
    height: auto;
    display: block;
  }
</style>
</head>
<body>
<header>
  <h1>panimg Visual Test Gallery</h1>
GALLERY_HEAD

# Insert dynamic summary
cat >> "$RESULTS_DIR/gallery.html" << EOF
  <div class="meta">Generated: ${TIMESTAMP}</div>
  <div class="summary">
    Total: ${TOTAL} |
    <span class="pass">Pass: ${PASS}</span> |
    <span class="fail">Fail: ${FAIL}</span>
  </div>
EOF

cat >> "$RESULTS_DIR/gallery.html" << 'GALLERY_MID'
</header>
<div class="grid">
GALLERY_MID

# Add each result image to gallery
for img in "$RESULTS_DIR"/*.png; do
    [[ -f "$img" ]] || continue
    fname=$(basename "$img")
    num="${fname%%_*}"

    # Check if there's a corresponding GIF (for animate/gif-speed)
    gif_name="${fname%.png}.gif"
    if [[ -f "$RESULTS_DIR/$gif_name" ]]; then
        cat >> "$RESULTS_DIR/gallery.html" << EOF
  <div class="card">
    <img src="${fname}" alt="${fname}" loading="lazy">
    <img src="${gif_name}" alt="${gif_name} (animated)" loading="lazy" style="border-top: 1px solid #333;">
  </div>
EOF
    else
        cat >> "$RESULTS_DIR/gallery.html" << EOF
  <div class="card">
    <img src="${fname}" alt="${fname}" loading="lazy">
  </div>
EOF
    fi
done

cat >> "$RESULTS_DIR/gallery.html" << 'GALLERY_TAIL'
</div>
</body>
</html>
GALLERY_TAIL

echo -e "${GREEN}Gallery generated: ${RESULTS_DIR}/gallery.html${NC}"

# ---------- Cleanup ----------

rm -rf "$TMP_DIR"

# ---------- Summary ----------

echo ""
echo -e "${CYAN}=== Summary ===${NC}"
echo -e "  Total:  ${TOTAL}"
echo -e "  ${GREEN}Pass:   ${PASS}${NC}"
if [[ $FAIL -gt 0 ]]; then
    echo -e "  ${RED}Fail:   ${FAIL}${NC}"
fi
echo ""
echo -e "Open the gallery:  ${YELLOW}open ${RESULTS_DIR}/gallery.html${NC}"
