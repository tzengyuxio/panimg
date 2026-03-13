# 支援格式

## 內建格式

以下格式開箱即用，無需額外的 feature flag：

| 格式 | 解碼 | 編碼 | 備註 |
|------|------|------|------|
| JPEG | 是 | 是 | 品質 1-100 |
| PNG | 是 | 是 | |
| WebP | 是 | 是 | 品質 1-100 |
| GIF | 是 | 是 | 支援動態 GIF |
| BMP | 是 | 是 | |
| TIFF | 是 | 是 | |
| QOI | 是 | 是 | |

## 選用編解碼器 Feature Flag

可在建構時透過 Cargo feature flag 啟用額外格式：

| 格式 | 解碼 | 編碼 | Feature Flag | 備註 |
|------|------|------|-------------|------|
| AVIF | 是 | 是 | `avif` | AVIF 編碼/解碼 |
| JPEG XL | 是 | 否 | `jxl` | 僅支援解碼 |
| SVG | 是 | 否 | `svg` | 僅支援柵格化 |
| PDF | 是 | 否 | `pdf` | 首頁柵格化；`--dpi` 選項 |
| HEIC | 是 | 否 | `heic` | 僅 macOS；需要系統 libheif ≥ 1.21 |

### 建構選用編解碼器

啟用個別編解碼器：

```bash
cargo build --release --features avif        # AVIF 編碼/解碼
cargo build --release --features jxl         # JPEG XL 解碼
cargo build --release --features svg         # SVG 柵格化
cargo build --release --features pdf         # PDF 柵格化
cargo build --release --features heic        # HEIC 解碼（僅 macOS）
```

一次啟用所有選用編解碼器：

```bash
cargo build --release --features all-codecs
```

透過 Cargo 安裝時：

```bash
cargo install panimg-cli --features avif
cargo install panimg-cli --features all-codecs
```

## 格式偵測

panimg 會從 `-o` / `--output` 指定的副檔名推斷輸出格式。使用 `batch convert` 時，請以 `--to` 旗標指定目標格式。

支援的副檔名：`.jpg` / `.jpeg`、`.png`、`.webp`、`.gif`、`.bmp`、`.tiff` / `.tif`、`.qoi`、`.avif`、`.jxl`、`.svg`（僅限輸入）、`.pdf`（僅限輸入）、`.heic` / `.heif`（僅限輸入，macOS）。
