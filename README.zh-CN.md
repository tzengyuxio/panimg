# panimg

[![CI](https://github.com/tzengyuxio/panimg/actions/workflows/ci.yml/badge.svg)](https://github.com/tzengyuxio/panimg/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-2021_edition-orange.svg)](https://www.rust-lang.org/)

[English](README.md) | [日本語](README.ja.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md)

使用 Rust 构建的下一代图片处理 CLI 工具。作为 ImageMagick 的现代替代方案，原生支持 AI 代理集成。

## 特性

- **31 个命令**涵盖格式转换、几何变换、滤镜、色彩调整、动画、合成、文字渲染等
- **结构化输出**：默认人类友好格式，`--format json` 输出机器可读的 JSON
- **干跑模式**：使用 `--dry-run` 预览操作结果，不实际执行
- **Schema 查询**：`--schema` 以 JSON 返回命令参数定义
- **能力发现**：`--capabilities` 列出所有支持的命令、格式和功能
- **管线引擎**：在单次读写中串接多个操作
- **批量处理**：以 glob 模式匹配多文件，并行处理
- **一致语法**：所有参数统一使用 `--key value` 格式
- **内存安全**：以 Rust 构建，核心管线零 unsafe 代码
- **高性能**：通过 `fast_image_resize` 实现 SIMD 加速缩放（[性能测试报告](benches/results/REPORT.md)）

## 安装

```bash
cargo install panimg-cli
```

或从源码构建：

```bash
git clone https://github.com/tzengyuxio/panimg.git
cd panimg
cargo build --release
# 可执行文件位于 target/release/panimg
```

### 可选编解码器

通过 feature flag 启用额外格式支持：

```bash
cargo build --release --features avif    # AVIF 编解码
cargo build --release --features jxl     # JPEG XL 解码
cargo build --release --features svg     # SVG 光栅化
cargo build --release --features all-codecs  # 以上全部
```

## 快速开始

```bash
# 显示图片信息
panimg info photo.jpg
panimg info photo.jpg --format json --fields width,height

# 格式转换
panimg convert photo.png -o photo.webp --quality 80

# 缩放
panimg resize photo.jpg --width 800 -o thumbnail.jpg

# 裁切、旋转、翻转
panimg crop photo.jpg --x 100 --y 100 --width 400 --height 300 -o cropped.jpg
panimg rotate photo.jpg --angle 90 -o rotated.jpg
panimg flip photo.jpg --direction horizontal -o flipped.jpg

# 色彩调整
panimg brightness photo.jpg --value 20 -o brighter.jpg
panimg saturate photo.jpg --factor 1.5 -o vivid.jpg
panimg sepia photo.jpg -o vintage.jpg

# 滤镜
panimg blur photo.jpg --sigma 3.0 -o blurred.jpg
panimg sharpen photo.jpg --sigma 1.5 -o sharp.jpg
panimg edge-detect photo.jpg -o edges.jpg

# 色彩效果
panimg tint photo.jpg --color red --strength 0.3 -o warm.jpg
panimg posterize photo.jpg --levels 4 -o poster.jpg

# 绘图
panimg draw photo.jpg --shape rect --x 10 --y 10 --width 100 --height 50 --color red -o annotated.jpg

# 文字渲染
panimg text photo.jpg --content "Hello World" --size 48 --color white --position center -o titled.jpg
panimg text photo.jpg --content "© 2026" --size 16 --color '#FFFFFF80' --position bottom-right -o stamped.jpg

# 合成
panimg overlay base.jpg --layer watermark.png --position bottom-right --opacity 0.5 -o watermarked.jpg

# GIF 动画
panimg frames animation.gif --output-dir ./frames
panimg animate 'frames/*.png' -o animation.gif --delay 100
panimg gif-speed animation.gif -o fast.gif --speed 2.0

# 图片比较
panimg diff before.png after.png -o diff.png

# 管线（多步骤操作）
panimg pipeline photo.jpg -o result.jpg --steps "resize --width 800 | blur --sigma 1.5 | grayscale"

# 批量处理
panimg batch convert 'photos/*.png' --output-dir ./webp --to webp --quality 80
panimg batch resize 'photos/*.jpg' --output-dir ./thumbs --width 200

# 干跑与 Schema
panimg resize photo.jpg --width 800 -o out.jpg --dry-run --format json
panimg resize --schema
panimg --capabilities --format json
```

## 命令一览

### 基本操作

| 命令 | 说明 |
|------|------|
| `info` | 显示图片 metadata 与属性 |
| `convert` | 图片格式转换 |
| `resize` | 缩放图片，支持多种适配模式 |
| `crop` | 裁切矩形区域 |
| `rotate` | 旋转 90、180 或 270 度 |
| `flip` | 水平或垂直翻转 |
| `auto-orient` | 根据 EXIF 方向标签自动旋转 |

### 色彩调整

| 命令 | 说明 |
|------|------|
| `brightness` | 调整亮度 |
| `contrast` | 调整对比度 |
| `hue-rotate` | 色相旋转 |
| `saturate` | 调整饱和度 |
| `grayscale` | 转为灰度 |
| `invert` | 反转色彩 |
| `sepia` | 应用复古棕褐色调 |
| `tint` | 以指定色彩着色 |
| `posterize` | 减少每通道色阶数 |

### 滤镜

| 命令 | 说明 |
|------|------|
| `blur` | 高斯模糊 |
| `sharpen` | 锐化（非锐化蒙版） |
| `edge-detect` | 边缘检测（Laplacian 核） |
| `emboss` | 浮雕效果 |

### 绘图与合成

| 命令 | 说明 |
|------|------|
| `draw` | 绘制图形（矩形、圆形、线段） |
| `text` | 绘制文字，支持内嵌或自定义字体 |
| `overlay` | 叠加合成图片 |
| `trim` | 自动裁切空白或相似色边框 |
| `diff` | 比较两张图片并可视化差异 |

### 动画

| 命令 | 说明 |
|------|------|
| `frames` | 从动态 GIF 提取单帧 |
| `animate` | 将多张图片组合成动态 GIF |
| `gif-speed` | 调整动画播放速度 |

### 工作流

| 命令 | 说明 |
|------|------|
| `pipeline` | 在单次读写中执行多步骤操作 |
| `batch` | 以 glob 模式批量处理多个文件 |

## 支持格式

| 格式 | 解码 | 编码 | 备注 |
|------|------|------|------|
| JPEG | Yes | Yes | 质量 1-100 |
| PNG | Yes | Yes | |
| WebP | Yes | Yes | 质量 1-100 |
| GIF | Yes | Yes | 支持动态 GIF |
| BMP | Yes | Yes | |
| TIFF | Yes | Yes | |
| QOI | Yes | Yes | |
| AVIF | Yes | Yes | 需启用 `avif` feature |
| JPEG XL | Yes | No | 需启用 `jxl` feature，仅解码 |
| SVG | Yes | No | 需启用 `svg` feature，仅光栅化 |

## AI 代理集成

panimg 专为 AI 代理和自动化脚本设计：

```bash
# 程序化发现能力
panimg --capabilities --format json

# 查询任意命令的参数 Schema
panimg resize --schema

# 预览操作，无副作用
panimg resize photo.jpg --width 800 -o out.jpg --dry-run --format json

# 所有输出均可为结构化 JSON
panimg info photo.jpg --format json --fields width,height,format
```

## 退出码

| 代码 | 含义 |
|------|------|
| 0 | 成功 |
| 1 | 一般错误 |
| 2 | 输入文件错误（文件未找到、权限不足、解码失败） |
| 3 | 输出问题（写入失败、文件已存在） |
| 4 | 不支持的格式 |
| 5 | 参数错误 |

## 错误输出

错误信息均为结构化格式，并包含可操作的建议：

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

## 许可证

本项目采用以下任一许可：

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

由您自行选择。
