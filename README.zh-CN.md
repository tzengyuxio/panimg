# panimg

[![CI](https://github.com/tzengyuxio/panimg/actions/workflows/ci.yml/badge.svg)](https://github.com/tzengyuxio/panimg/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-2021_edition-orange.svg)](https://www.rust-lang.org/)

[English](README.md) | [日本語](README.ja.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md)

图片处理的瑞士刀——为人类与 AI 代理而生。

## 特性

- **丰富的命令集**涵盖格式转换、缩放、裁切、旋转、色彩调整、滤镜、文字、合成、动画、压缩等
- **管线引擎**——在单次读写中串接多个操作
- **批量处理**——以 glob 模式匹配多文件，并行处理
- **AI 代理友好**——结构化 JSON 输出、`--dry-run`、`--schema`、`--capabilities` 支持程序化使用
- **快速且安全**——以 Rust 构建，SIMD 加速缩放，核心管线零 unsafe 代码（[性能测试报告](benches/results/REPORT.md)）

## 安装

### Homebrew (macOS / Linux)

```bash
brew install tzengyuxio/tap/panimg
```

### Cargo

```bash
cargo install panimg-cli
```

### 从源码构建

```bash
git clone https://github.com/tzengyuxio/panimg.git
cd panimg
cargo build --release
```

可选编解码器（AVIF、JPEG XL、SVG）请参阅[支持格式](docs/zh-CN/formats.md)。

## 快速开始

```bash
# 查询图片信息
panimg info photo.jpg --format json

# 格式转换
panimg convert photo.png -o photo.webp --quality 80

# 缩放（指定适配模式）
panimg resize photo.jpg --width 800 --height 600 --fit cover -o thumb.jpg

# 色彩调整
panimg saturate photo.jpg --factor 1.5 -o vivid.jpg

# 应用滤镜
panimg blur photo.jpg --sigma 3.0 -o blurred.jpg

# 添加水印文字
panimg text photo.jpg --content "© 2026" --size 24 --color white --position bottom-right -o stamped.jpg

# 智能压缩
panimg tiny photo.png -o compressed.png

# 多步骤管线
panimg pipeline photo.jpg -o result.jpg --steps "resize --width 800 | blur --sigma 1.5 | grayscale"

# 批量处理
panimg batch convert 'photos/*.png' --output-dir ./webp --to webp --quality 80
```

完整用法与示例请参阅[命令参考](docs/zh-CN/commands.md)。

## 命令一览

| 命令 | 说明 |
|------|------|
| `info` | 显示图片 metadata 与属性 |
| `convert` | 图片格式转换 |
| `tiny` | 智能压缩（类似 TinyPNG） |
| `resize` | 缩放图片，支持多种适配模式 |
| `crop` | 裁切矩形区域 |
| `rotate` | 旋转 90、180 或 270 度 |
| `flip` | 水平或垂直翻转 |
| `auto-orient` | 根据 EXIF 方向标签自动旋转 |
| `brightness` | 调整亮度 |
| `contrast` | 调整对比度 |
| `hue-rotate` | 色相旋转 |
| `saturate` | 调整饱和度 |
| `grayscale` | 转为灰度 |
| `invert` | 反转色彩 |
| `sepia` | 应用复古棕褐色调 |
| `tint` | 以指定色彩着色 |
| `posterize` | 减少每通道色阶数 |
| `blur` | 高斯模糊 |
| `sharpen` | 锐化（非锐化蒙版） |
| `edge-detect` | 边缘检测（Laplacian 核） |
| `emboss` | 浮雕效果 |
| `tilt-shift` | 移轴摄影（微缩模型）效果 |
| `draw` | 绘制图形（矩形、圆形、线段） |
| `text` | 绘制文字，支持内嵌或自定义字体 |
| `overlay` | 叠加合成图片 |
| `trim` | 自动裁切空白或相似色边框 |
| `diff` | 比较两张图片并可视化差异 |
| `frames` | 从动态 GIF 提取单帧 |
| `animate` | 将多张图片组合成动态 GIF |
| `gif-speed` | 调整动画播放速度 |
| `pipeline` | 在单次读写中执行多步骤操作 |
| `batch` | 以 glob 模式批量处理多个文件 |

完整用法与示例请参阅[命令参考](docs/zh-CN/commands.md)。

## AI 代理集成

panimg 支持程序化发现与结构化输出，适用于 AI 代理和自动化脚本：

```bash
panimg --capabilities --format json   # 发现所有命令与格式
panimg resize --schema                # 获取参数定义（JSON）
panimg resize photo.jpg --width 800 -o out.jpg --dry-run --format json  # 无副作用预览
```

退出码、错误格式与集成模式请参阅 [AI 集成指南](docs/zh-CN/ai-integration.md)。

## 文档

- [命令参考](docs/zh-CN/commands.md) — 完整命令用法与示例
- [支持格式](docs/zh-CN/formats.md) — 格式支持表与可选编解码器
- [AI 集成指南](docs/zh-CN/ai-integration.md) — 结构化输出、Schema、Dry-run、退出码

## 许可证

本项目采用以下任一许可：

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

由您自行选择。
