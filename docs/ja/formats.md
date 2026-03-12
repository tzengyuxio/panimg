# 対応フォーマット

## ビルトインフォーマット

以下のフォーマットは追加のフィーチャーフラグなしでそのまま利用できます：

| フォーマット | デコード | エンコード | 備考 |
|--------------|----------|------------|------|
| JPEG | 対応 | 対応 | 品質 1-100 |
| PNG | 対応 | 対応 | |
| WebP | 対応 | 対応 | 品質 1-100 |
| GIF | 対応 | 対応 | アニメーション GIF 対応 |
| BMP | 対応 | 対応 | |
| TIFF | 対応 | 対応 | |
| QOI | 対応 | 対応 | |

## オプションコーデックのフィーチャーフラグ

追加フォーマットは Cargo のフィーチャーフラグを使用してビルド時に有効化できます：

| フォーマット | デコード | エンコード | フィーチャーフラグ | 備考 |
|--------------|----------|------------|-------------------|------|
| AVIF | 対応 | 対応 | `avif` | AVIF エンコード/デコード |
| JPEG XL | 対応 | 非対応 | `jxl` | デコードのみ |
| SVG | 対応 | 非対応 | `svg` | ラスタライズのみ |

### オプションコーデックを含めたビルド

個別のコーデックを有効化：

```bash
cargo build --release --features avif        # AVIF エンコード/デコード
cargo build --release --features jxl         # JPEG XL デコード
cargo build --release --features svg         # SVG ラスタライズ
```

すべてのオプションコーデックを一括で有効化：

```bash
cargo build --release --features all-codecs
```

Cargo 経由でインストールする場合：

```bash
cargo install panimg-cli --features avif
cargo install panimg-cli --features all-codecs
```

## フォーマット検出

panimg は `-o` / `--output` で指定されたファイル拡張子から出力フォーマットを判定します。`batch convert` の場合は `--to` フラグでターゲットフォーマットを指定してください。

対応拡張子：`.jpg` / `.jpeg`、`.png`、`.webp`、`.gif`、`.bmp`、`.tiff` / `.tif`、`.qoi`、`.avif`、`.jxl`、`.svg`（入力のみ）。
