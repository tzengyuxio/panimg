# panimg

[![CI](https://github.com/tzengyuxio/panimg/actions/workflows/ci.yml/badge.svg)](https://github.com/tzengyuxio/panimg/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-2021_edition-orange.svg)](https://www.rust-lang.org/)

[English](README.md) | [日本語](README.ja.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md)

圖片處理的瑞士刀——為人類與 AI 代理而生。

## 特色

- **31 個命令**涵蓋格式轉換、縮放、裁切、旋轉、色彩調整、濾鏡、文字、合成、動畫等
- **管線引擎**——在單次讀寫中串接多個操作
- **批次處理**——以 glob 模式匹配多檔案，平行處理
- **AI 代理友善**——結構化 JSON 輸出、`--dry-run`、`--schema`、`--capabilities` 支援程式化使用
- **快速且安全**——以 Rust 建構，SIMD 加速縮放，核心管線零 unsafe 程式碼（[效能測試報告](benches/results/REPORT.md)）

## 安裝

### Homebrew (macOS / Linux)

```bash
brew install tzengyuxio/tap/panimg
```

### Cargo

```bash
cargo install panimg-cli
```

### 從原始碼建構

```bash
git clone https://github.com/tzengyuxio/panimg.git
cd panimg
cargo build --release
```

選用編解碼器（AVIF、JPEG XL、SVG）請參閱 [Supported Formats](docs/formats.md)。

## 快速開始

```bash
# 查詢圖片資訊
panimg info photo.jpg --format json

# 格式轉換
panimg convert photo.png -o photo.webp --quality 80

# 縮放（指定適配模式）
panimg resize photo.jpg --width 800 --height 600 --fit cover -o thumb.jpg

# 色彩調整
panimg saturate photo.jpg --factor 1.5 -o vivid.jpg

# 套用濾鏡
panimg blur photo.jpg --sigma 3.0 -o blurred.jpg

# 加入浮水印文字
panimg text photo.jpg --content "© 2026" --size 24 --color white --position bottom-right -o stamped.jpg

# 多步驟管線
panimg pipeline photo.jpg -o result.jpg --steps "resize --width 800 | blur --sigma 1.5 | grayscale"

# 批次處理
panimg batch convert 'photos/*.png' --output-dir ./webp --to webp --quality 80
```

完整 31 個命令的用法與範例請參閱 [Command Reference](docs/commands.md)。

## 命令一覽

| 命令 | 說明 |
|------|------|
| `info` | 顯示圖片 metadata 與屬性 |
| `convert` | 圖片格式轉換 |
| `resize` | 縮放圖片，支援多種適配模式 |
| `crop` | 裁切矩形區域 |
| `rotate` | 旋轉 90、180 或 270 度 |
| `flip` | 水平或垂直翻轉 |
| `auto-orient` | 依據 EXIF 方向標籤自動旋轉 |
| `brightness` | 調整亮度 |
| `contrast` | 調整對比度 |
| `hue-rotate` | 色相旋轉 |
| `saturate` | 調整飽和度 |
| `grayscale` | 轉為灰階 |
| `invert` | 反轉色彩 |
| `sepia` | 套用復古棕褐色調 |
| `tint` | 以指定色彩著色 |
| `posterize` | 減少每通道色階數 |
| `blur` | 高斯模糊 |
| `sharpen` | 銳化（非銳化遮罩） |
| `edge-detect` | 邊緣偵測（Laplacian 核） |
| `emboss` | 浮雕效果 |
| `draw` | 繪製圖形（矩形、圓形、線段） |
| `text` | 繪製文字，支援內嵌或自訂字型 |
| `overlay` | 疊加合成圖片 |
| `trim` | 自動裁切空白或相似色邊框 |
| `diff` | 比較兩張圖片並視覺化差異 |
| `frames` | 從動態 GIF 提取個別幀 |
| `animate` | 將多張圖片組合成動態 GIF |
| `gif-speed` | 調整動畫播放速度 |
| `pipeline` | 在單次讀寫中執行多步驟操作 |
| `batch` | 以 glob 模式批次處理多個檔案 |

完整用法與範例請參閱 [Command Reference](docs/commands.md)。

## AI 代理整合

panimg 支援程式化探索與結構化輸出，適用於 AI 代理和自動化腳本：

```bash
panimg --capabilities --format json   # 探索所有命令與格式
panimg resize --schema                # 取得參數定義（JSON）
panimg resize photo.jpg --width 800 -o out.jpg --dry-run --format json  # 無副作用預覽
```

Exit codes、錯誤格式與整合模式請參閱 [AI Integration Guide](docs/ai-integration.md)。

## 文件

- [Command Reference](docs/commands.md) — 完整 31 個命令的用法與範例
- [Supported Formats](docs/formats.md) — 格式支援表與選用編解碼器
- [AI Integration Guide](docs/ai-integration.md) — 結構化輸出、Schema、Dry-run、Exit codes

## 授權條款

本專案採用以下任一授權：

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

由您自行選擇。
