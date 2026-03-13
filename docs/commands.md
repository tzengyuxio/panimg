# Command Reference

panimg provides a rich set of commands covering image processing, color manipulation, filters, drawing, animation, optimization, and workflow automation. All commands follow a consistent `panimg <command> <input> [options]` syntax.

## Global Options

Every command supports these flags:

| Flag | Description |
|------|-------------|
| `-o`, `--output` | Output file path |
| `--format` | Output format: `text` (default) or `json` |
| `--dry-run` | Preview the operation without writing any files |
| `--schema` | Print the command's parameter definitions as JSON |

Additionally, the following global flags are available:

```bash
panimg --capabilities              # List all supported commands, formats, and features
panimg --capabilities --format json  # Same, as structured JSON
```

---

## Basic Operations

### `info`

Show image metadata and properties.

```bash
panimg info photo.jpg
panimg info photo.jpg --format json
panimg info photo.jpg --format json --fields width,height,format
```

| Option | Description |
|--------|-------------|
| `--format` | Output format: `text` or `json` |
| `--fields` | Comma-separated list of fields to include (JSON mode). Includes `icc_profile` when `icc` feature is enabled |

### `convert`

Convert image between formats. The output format is inferred from the file extension.

```bash
panimg convert photo.png -o photo.webp
panimg convert photo.png -o photo.webp --quality 80
panimg convert photo.jpg --convert-profile display-p3 -o photo-p3.jpg
```

| Option | Description |
|--------|-------------|
| `-o`, `--output` | Output file path (required) |
| `--quality` | Quality level 1-100 (for JPEG, WebP, AVIF) |
| `--convert-profile` | Convert to a target color space: `srgb`, `adobe-rgb`, `display-p3` (requires `icc` feature) |
| `--strip` | Strip metadata from output |

### `resize`

Resize an image with multiple fit modes.

```bash
panimg resize photo.jpg --width 800 -o thumbnail.jpg
panimg resize photo.jpg --width 800 --height 600 --fit cover -o thumb.jpg
```

| Option | Description |
|--------|-------------|
| `--width` | Target width in pixels |
| `--height` | Target height in pixels |
| `--fit` | Fit mode: `contain` (default), `cover`, `fill`, `inside`, `outside` |
| `-o`, `--output` | Output file path |

### `crop`

Crop a rectangular region from the image.

```bash
panimg crop photo.jpg --x 100 --y 100 --width 400 --height 300 -o cropped.jpg
```

| Option | Description |
|--------|-------------|
| `--x` | Left offset in pixels |
| `--y` | Top offset in pixels |
| `--width` | Crop width in pixels |
| `--height` | Crop height in pixels |
| `-o`, `--output` | Output file path |

### `rotate`

Rotate by any angle. 90, 180, and 270 degrees use a fast lossless path; arbitrary angles (e.g. 45, 30.5) use bilinear interpolation.

```bash
panimg rotate photo.jpg --angle 90 -o rotated.jpg
panimg rotate photo.jpg --angle 45 --background white -o rotated.jpg
```

| Option | Description |
|--------|-------------|
| `--angle` | Rotation angle: `90`, `180`, `270`, `left`, `right`, or any numeric value (e.g. `45`, `30.5`) |
| `--background` | Fill color for areas exposed by arbitrary-angle rotation. Hex (`#FF0000`), RGB (`255,0,0`), or named (`white`, `transparent`, etc.). Default: `transparent` for formats with alpha, `white` for JPEG/BMP |
| `-o`, `--output` | Output file path |

### `flip`

Mirror horizontally or vertically.

```bash
panimg flip photo.jpg --direction horizontal -o flipped.jpg
```

| Option | Description |
|--------|-------------|
| `--direction` | Flip direction: `horizontal` or `vertical` |
| `-o`, `--output` | Output file path |

### `auto-orient`

Auto-rotate based on EXIF orientation tag, then strip the tag.

```bash
panimg auto-orient photo.jpg -o oriented.jpg
```

---

## Color Adjustments

### `brightness`

Adjust image brightness.

```bash
panimg brightness photo.jpg --value 20 -o brighter.jpg
panimg brightness photo.jpg --value -10 -o darker.jpg
```

| Option | Description |
|--------|-------------|
| `--value` | Brightness adjustment value (positive = brighter, negative = darker) |

### `contrast`

Adjust image contrast.

```bash
panimg contrast photo.jpg --value 1.5 -o enhanced.jpg
```

| Option | Description |
|--------|-------------|
| `--value` | Contrast multiplier (1.0 = no change, >1 = more contrast) |

### `hue-rotate`

Rotate image hue.

```bash
panimg hue-rotate photo.jpg --degrees 90 -o shifted.jpg
```

| Option | Description |
|--------|-------------|
| `--degrees` | Hue rotation in degrees |

### `saturate`

Adjust color saturation.

```bash
panimg saturate photo.jpg --factor 1.5 -o vivid.jpg
panimg saturate photo.jpg --factor 0.5 -o muted.jpg
```

| Option | Description |
|--------|-------------|
| `--factor` | Saturation multiplier (1.0 = no change, 0 = grayscale, >1 = more saturated) |

### `grayscale`

Convert to grayscale.

```bash
panimg grayscale photo.jpg -o bw.jpg
```

### `invert`

Invert (negate) all colors.

```bash
panimg invert photo.jpg -o inverted.jpg
```

### `sepia`

Apply sepia tone effect.

```bash
panimg sepia photo.jpg -o vintage.jpg
```

### `tint`

Tint image with a specified color.

```bash
panimg tint photo.jpg --color red --strength 0.3 -o warm.jpg
panimg tint photo.jpg --color '#FF6600' --strength 0.5 -o orange.jpg
```

| Option | Description |
|--------|-------------|
| `--color` | Tint color (name or hex) |
| `--strength` | Tint strength from 0.0 to 1.0 |

### `posterize`

Reduce color levels per channel.

```bash
panimg posterize photo.jpg --levels 4 -o poster.jpg
```

| Option | Description |
|--------|-------------|
| `--levels` | Number of color levels per channel |

---

## Filters

### `blur`

Apply Gaussian blur.

```bash
panimg blur photo.jpg --sigma 3.0 -o blurred.jpg
```

| Option | Description |
|--------|-------------|
| `--sigma` | Blur radius (higher = more blur) |

### `sharpen`

Sharpen using unsharp mask.

```bash
panimg sharpen photo.jpg --sigma 1.5 -o sharp.jpg
```

| Option | Description |
|--------|-------------|
| `--sigma` | Sharpen intensity |

### `edge-detect`

Detect edges using Laplacian kernel.

```bash
panimg edge-detect photo.jpg -o edges.jpg
```

### `emboss`

Apply emboss effect.

```bash
panimg emboss photo.jpg -o embossed.jpg
```

### `tilt-shift`

Apply a miniature/diorama tilt-shift effect. Keeps a horizontal band in focus while progressively blurring regions above and below.

```bash
panimg tilt-shift photo.jpg -o miniature.jpg
panimg tilt-shift photo.jpg --sigma 12 --focus-position 0.4 --focus-width 0.2 --saturation 1.3 -o miniature.jpg
```

| Option | Description |
|--------|-------------|
| `--sigma` | Out-of-focus blur strength (default: 8.0) |
| `--focus-position` | Vertical center of focus band, 0=top, 1=bottom (default: 0.5) |
| `--focus-width` | Height of focus band as fraction of image height (default: 0.15) |
| `--transition` | Transition zone width as fraction of image height (default: 0.2) |
| `--saturation` | Saturation multiplier, >1 enhances miniature look (default: 1.0) |

### `smart-crop`

Automatically select the best crop region based on image content. Supports entropy (Shannon entropy, prefers high-detail areas) and attention (edges + saturation + skin tone weighted) strategies.

```bash
panimg smart-crop photo.jpg -o cropped.jpg --width 400 --height 300
panimg smart-crop photo.jpg -o cropped.jpg --width 400 --height 300 --strategy attention
panimg smart-crop photo.jpg -o cropped.jpg --width 400 --height 300 --strategy entropy --step 5
```

| Option | Description |
|--------|-------------|
| `--width` | Crop width in pixels (required) |
| `--height` | Crop height in pixels (required) |
| `--strategy` | Scoring strategy: `entropy` or `attention` (default: entropy) |
| `--step` | Search step size in pixels (default: auto) |

---

## Drawing & Compositing

### `draw`

Draw shapes on an image.

```bash
panimg draw photo.jpg --shape rect --x 10 --y 10 --width 100 --height 50 --color red -o annotated.jpg
panimg draw photo.jpg --shape circle --cx 200 --cy 200 --radius 50 --color blue --fill -o marked.jpg
panimg draw photo.jpg --shape line --x1 0 --y1 0 --x2 100 --y2 100 --color white -o lined.jpg
```

| Option | Description |
|--------|-------------|
| `--shape` | Shape type: `rect`, `circle`, `line` |
| `--x`, `--y` | Position (rect) |
| `--cx`, `--cy` | Center position (circle) |
| `--x1`, `--y1`, `--x2`, `--y2` | Start/end points (line) |
| `--width`, `--height` | Dimensions (rect) |
| `--radius` | Radius (circle) |
| `--color` | Shape color (name or hex) |
| `--fill` | Fill the shape (flag) |

### `text`

Draw text with embedded or custom fonts.

```bash
panimg text photo.jpg --content "Hello World" --size 48 --color white --position center -o titled.jpg
panimg text photo.jpg --content "© 2026" --size 16 --color '#FFFFFF80' --position bottom-right -o stamped.jpg
```

| Option | Description |
|--------|-------------|
| `--content` | Text string to render |
| `--size` | Font size in pixels |
| `--color` | Text color (name, hex, or hex with alpha) |
| `--position` | Placement: `center`, `top-left`, `top-right`, `bottom-left`, `bottom-right` |
| `--font` | Path to a custom font file (optional) |

### `overlay`

Composite one image on another.

```bash
panimg overlay base.jpg --layer watermark.png --position bottom-right --opacity 0.5 -o watermarked.jpg
```

| Option | Description |
|--------|-------------|
| `--layer` | Path to the overlay image |
| `--position` | Placement position |
| `--opacity` | Overlay opacity from 0.0 to 1.0 |

### `trim`

Auto-crop whitespace or similar-colored borders.

```bash
panimg trim photo.jpg -o trimmed.jpg
```

### `diff`

Compare two images and visualize differences.

```bash
panimg diff before.png after.png -o diff.png
```

---

## Animation

### `frames`

Extract individual frames from an animated GIF.

```bash
panimg frames animation.gif --output-dir ./frames
```

| Option | Description |
|--------|-------------|
| `--output-dir` | Directory to save extracted frames |

### `animate`

Assemble images into an animated GIF.

```bash
panimg animate 'frames/*.png' -o animation.gif --delay 100
```

| Option | Description |
|--------|-------------|
| `--delay` | Frame delay in milliseconds |
| `-o`, `--output` | Output GIF file path |

### `gif-speed`

Change animation playback speed.

```bash
panimg gif-speed animation.gif -o fast.gif --speed 2.0
panimg gif-speed animation.gif -o slow.gif --speed 0.5
```

| Option | Description |
|--------|-------------|
| `--speed` | Speed multiplier (2.0 = twice as fast, 0.5 = half speed) |

---

## Optimization

### `tiny`

Smart image compression — automatically selects the best strategy per format (like TinyPNG).

- **PNG**: lossy quantization (imagequant) + lossless optimization (oxipng)
- **JPEG**: quality-controlled re-encoding (default quality 75)
- **WebP**: quality-controlled encoding (default quality 75)
- **AVIF**: quality-controlled encoding (default quality 68, requires `avif` feature)

```bash
panimg tiny photo.png                           # → photo_tiny.png
panimg tiny photo.png -o compressed.png         # specify output
panimg tiny photo.jpg --quality 60              # custom quality
panimg tiny icon.png --lossless                 # PNG: lossless optimization only
panimg tiny photo.png --max-colors 128          # PNG: limit palette colors
panimg tiny photo.png --strip                   # strip metadata
panimg batch tiny 'photos/*.png' --output-dir compressed/  # batch mode
```

| Option | Description |
|--------|-------------|
| `-o`, `--output` | Output file path (default: `{stem}_tiny.{ext}`) |
| `--quality` | Compression quality 1-100 (JPEG, WebP, AVIF) |
| `--max-colors` | PNG: max palette colors for quantization (2-256, default 256) |
| `--lossless` | PNG: skip quantization, only lossless optimization |
| `--strip` | Strip metadata from output |

---

## Workflow

### `pipeline`

Run multiple operations in a single read/write pass.

```bash
panimg pipeline photo.jpg -o result.jpg --steps "resize --width 800 | blur --sigma 1.5 | grayscale"
panimg pipeline photo.jpg -o result.jpg --steps "brightness --value 10 | contrast --value 1.2 | sharpen --sigma 1.0"
```

| Option | Description |
|--------|-------------|
| `--steps` | Pipe-separated list of operations |
| `-o`, `--output` | Output file path |

### `batch`

Process multiple files with glob patterns and parallel execution.

```bash
panimg batch convert 'photos/*.png' --output-dir ./webp --to webp --quality 80
panimg batch resize 'photos/*.jpg' --output-dir ./thumbs --width 200
```

| Option | Description |
|--------|-------------|
| `--output-dir` | Directory for output files |
| `--to` | Target format (for `batch convert`) |
| Other options | Command-specific options are passed through |
