# panimg

[![CI](https://github.com/tzengyuxio/panimg/actions/workflows/ci.yml/badge.svg)](https://github.com/tzengyuxio/panimg/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-2021_edition-orange.svg)](https://www.rust-lang.org/)

[English](README.md) | [日本語](README.ja.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md)

圖片處理的瑞士刀——為人類與 AI 代理而生。

## 特色

- **31 個命令**涵蓋格式轉換、幾何變換、濾鏡、色彩調整、動畫、合成、文字渲染等
- **結構化輸出**：預設人類友善格式，`--format json` 輸出機器可讀的 JSON
- **乾跑模式**：使用 `--dry-run` 預覽操作結果，不實際執行
- **Schema 查詢**：`--schema` 以 JSON 回傳命令參數定義
- **能力探索**：`--capabilities` 列出所有支援的命令、格式和功能
- **管線引擎**：在單次讀寫中串接多個操作
- **批次處理**：以 glob 模式匹配多檔案，平行處理
- **一致語法**：所有參數統一使用 `--key value` 格式
- **記憶體安全**：以 Rust 建構，核心管線零 unsafe 程式碼
- **高效能**：透過 `fast_image_resize` 實現 SIMD 加速縮放

## 安裝

```bash
cargo install panimg-cli
```

或從原始碼建構：

```bash
git clone https://github.com/tzengyuxio/panimg.git
cd panimg
cargo build --release
# 執行檔位於 target/release/panimg
```

### 選用編解碼器

透過 feature flag 啟用額外格式支援：

```bash
cargo build --release --features avif    # AVIF 編解碼
cargo build --release --features jxl     # JPEG XL 解碼
cargo build --release --features svg     # SVG 光柵化
cargo build --release --features all-codecs  # 以上全部
```

## 快速開始

```bash
# 顯示圖片資訊
panimg info photo.jpg
panimg info photo.jpg --format json --fields width,height

# 格式轉換
panimg convert photo.png -o photo.webp --quality 80

# 縮放
panimg resize photo.jpg --width 800 -o thumbnail.jpg

# 裁切、旋轉、翻轉
panimg crop photo.jpg --x 100 --y 100 --width 400 --height 300 -o cropped.jpg
panimg rotate photo.jpg --angle 90 -o rotated.jpg
panimg flip photo.jpg --direction horizontal -o flipped.jpg

# 色彩調整
panimg brightness photo.jpg --value 20 -o brighter.jpg
panimg saturate photo.jpg --factor 1.5 -o vivid.jpg
panimg sepia photo.jpg -o vintage.jpg

# 濾鏡
panimg blur photo.jpg --sigma 3.0 -o blurred.jpg
panimg sharpen photo.jpg --sigma 1.5 -o sharp.jpg
panimg edge-detect photo.jpg -o edges.jpg

# 色彩效果
panimg tint photo.jpg --color red --strength 0.3 -o warm.jpg
panimg posterize photo.jpg --levels 4 -o poster.jpg

# 繪圖
panimg draw photo.jpg --shape rect --x 10 --y 10 --width 100 --height 50 --color red -o annotated.jpg

# 文字渲染
panimg text photo.jpg --content "Hello World" --size 48 --color white --position center -o titled.jpg
panimg text photo.jpg --content "© 2026" --size 16 --color '#FFFFFF80' --position bottom-right -o stamped.jpg

# 合成
panimg overlay base.jpg --layer watermark.png --position bottom-right --opacity 0.5 -o watermarked.jpg

# GIF 動畫
panimg frames animation.gif --output-dir ./frames
panimg animate 'frames/*.png' -o animation.gif --delay 100
panimg gif-speed animation.gif -o fast.gif --speed 2.0

# 圖片比較
panimg diff before.png after.png -o diff.png

# 管線（多步驟操作）
panimg pipeline photo.jpg -o result.jpg --steps "resize --width 800 | blur --sigma 1.5 | grayscale"

# 批次處理
panimg batch convert 'photos/*.png' --output-dir ./webp --to webp --quality 80
panimg batch resize 'photos/*.jpg' --output-dir ./thumbs --width 200

# 乾跑與 Schema
panimg resize photo.jpg --width 800 -o out.jpg --dry-run --format json
panimg resize --schema
panimg --capabilities --format json
```

## 命令一覽

### 基本操作

| 命令 | 說明 |
|------|------|
| `info` | 顯示圖片 metadata 與屬性 |
| `convert` | 圖片格式轉換 |
| `resize` | 縮放圖片，支援多種適配模式 |
| `crop` | 裁切矩形區域 |
| `rotate` | 旋轉 90、180 或 270 度 |
| `flip` | 水平或垂直翻轉 |
| `auto-orient` | 依據 EXIF 方向標籤自動旋轉 |

### 色彩調整

| 命令 | 說明 |
|------|------|
| `brightness` | 調整亮度 |
| `contrast` | 調整對比度 |
| `hue-rotate` | 色相旋轉 |
| `saturate` | 調整飽和度 |
| `grayscale` | 轉為灰階 |
| `invert` | 反轉色彩 |
| `sepia` | 套用復古棕褐色調 |
| `tint` | 以指定色彩著色 |
| `posterize` | 減少每通道色階數 |

### 濾鏡

| 命令 | 說明 |
|------|------|
| `blur` | 高斯模糊 |
| `sharpen` | 銳化（非銳化遮罩） |
| `edge-detect` | 邊緣偵測（Laplacian 核） |
| `emboss` | 浮雕效果 |

### 繪圖與合成

| 命令 | 說明 |
|------|------|
| `draw` | 繪製圖形（矩形、圓形、線段） |
| `text` | 繪製文字，支援內嵌或自訂字型 |
| `overlay` | 疊加合成圖片 |
| `trim` | 自動裁切空白或相似色邊框 |
| `diff` | 比較兩張圖片並視覺化差異 |

### 動畫

| 命令 | 說明 |
|------|------|
| `frames` | 從動態 GIF 提取個別幀 |
| `animate` | 將多張圖片組合成動態 GIF |
| `gif-speed` | 調整動畫播放速度 |

### 工作流程

| 命令 | 說明 |
|------|------|
| `pipeline` | 在單次讀寫中執行多步驟操作 |
| `batch` | 以 glob 模式批次處理多個檔案 |

## 支援格式

| 格式 | 解碼 | 編碼 | 備註 |
|------|------|------|------|
| JPEG | Yes | Yes | 品質 1-100 |
| PNG | Yes | Yes | |
| WebP | Yes | Yes | 品質 1-100 |
| GIF | Yes | Yes | 支援動態 GIF |
| BMP | Yes | Yes | |
| TIFF | Yes | Yes | |
| QOI | Yes | Yes | |
| AVIF | Yes | Yes | 需啟用 `avif` feature |
| JPEG XL | Yes | No | 需啟用 `jxl` feature，僅解碼 |
| SVG | Yes | No | 需啟用 `svg` feature，僅光柵化 |

## AI 代理整合

panimg 專為 AI 代理和自動化腳本設計：

```bash
# 程式化探索能力
panimg --capabilities --format json

# 查詢任何命令的參數 Schema
panimg resize --schema

# 預覽操作，無副作用
panimg resize photo.jpg --width 800 -o out.jpg --dry-run --format json

# 所有輸出皆可為結構化 JSON
panimg info photo.jpg --format json --fields width,height,format
```

## 結束碼

| 代碼 | 意義 |
|------|------|
| 0 | 成功 |
| 1 | 一般錯誤 |
| 2 | 輸入檔案錯誤（找不到檔案、權限不足、解碼失敗） |
| 3 | 輸出問題（寫入失敗、檔案已存在） |
| 4 | 不支援的格式 |
| 5 | 參數錯誤 |

## 錯誤輸出

錯誤訊息皆為結構化格式，並包含可行的建議：

```bash
$ panimg convert missing.png out.webp
error: file not found: missing.png
  hint: check that the file path is correct

$ panimg convert missing.png out.webp --format json
{
  "error": "file_not_found",
  "path": "missing.png",
  "suggestion": "check that the file path is correct"
}
```

## 授權條款

本專案採用以下任一授權：

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

由您自行選擇。
