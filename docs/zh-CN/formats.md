# 支持的格式

## 内置格式

以下格式开箱即用，无需额外的 feature flag：

| 格式 | 解码 | 编码 | 备注 |
|------|------|------|------|
| JPEG | 是 | 是 | 质量 1-100 |
| PNG | 是 | 是 | |
| WebP | 是 | 是 | 质量 1-100 |
| GIF | 是 | 是 | 支持动态 GIF |
| BMP | 是 | 是 | |
| TIFF | 是 | 是 | |
| QOI | 是 | 是 | |

## 可选编解码器 Feature Flag

可在构建时通过 Cargo feature flag 启用额外格式：

| 格式 | 解码 | 编码 | Feature Flag | 备注 |
|------|------|------|-------------|------|
| AVIF | 是 | 是 | `avif` | AVIF 编码/解码 |
| JPEG XL | 是 | 否 | `jxl` | 仅解码 |
| SVG | 是 | 否 | `svg` | 仅光栅化 |

### 构建可选编解码器

启用单个编解码器：

```bash
cargo build --release --features avif        # AVIF 编码/解码
cargo build --release --features jxl         # JPEG XL 解码
cargo build --release --features svg         # SVG 光栅化
```

一次启用所有可选编解码器：

```bash
cargo build --release --features all-codecs
```

通过 Cargo 安装时：

```bash
cargo install panimg-cli --features avif
cargo install panimg-cli --features all-codecs
```

## 格式检测

panimg 根据 `-o` / `--output` 中指定的文件扩展名确定输出格式。对于 `batch convert`，使用 `--to` 标志指定目标格式。

支持的扩展名：`.jpg` / `.jpeg`、`.png`、`.webp`、`.gif`、`.bmp`、`.tiff` / `.tif`、`.qoi`、`.avif`、`.jxl`、`.svg`（仅输入）。
