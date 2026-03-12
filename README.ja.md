# panimg

[![CI](https://github.com/tzengyuxio/panimg/actions/workflows/ci.yml/badge.svg)](https://github.com/tzengyuxio/panimg/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-2021_edition-orange.svg)](https://www.rust-lang.org/)

[English](README.md) | [日本語](README.ja.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md)

Rust で構築された次世代の画像処理 CLI ツール。AI エージェントとのネイティブ連携を備えた、ImageMagick のモダンな代替ツールです。

## 特徴

- **31 のコマンド**でフォーマット変換、幾何変換、フィルター、色調整、アニメーション、合成、テキスト描画などをカバー
- **構造化出力**：デフォルトは人間が読みやすい形式、`--format json` で機械可読な JSON を出力
- **ドライラン対応**：`--dry-run` で実行前に操作結果をプレビュー
- **スキーマ照会**：`--schema` でコマンドのパラメータ定義を JSON で取得
- **機能ディスカバリ**：`--capabilities` で対応コマンド・フォーマット・機能を一覧表示
- **パイプラインエンジン**：単一の読み書きで複数操作を連鎖実行
- **バッチ処理**：glob パターンによるマルチファイル処理を並列実行
- **統一構文**：すべてのフラグは `--key value` 形式
- **メモリ安全**：Rust で構築、コアパイプラインに unsafe コードなし
- **高速**：`fast_image_resize` による SIMD アクセラレーテッドリサイズ（[ベンチマーク](benches/results/REPORT.md)）

## インストール

```bash
cargo install panimg-cli
```

またはソースからビルド：

```bash
git clone https://github.com/tzengyuxio/panimg.git
cd panimg
cargo build --release
# バイナリは target/release/panimg に生成されます
```

### オプションコーデック

feature フラグで追加フォーマットを有効化：

```bash
cargo build --release --features avif    # AVIF エンコード/デコード
cargo build --release --features jxl     # JPEG XL デコード
cargo build --release --features svg     # SVG ラスタライズ
cargo build --release --features all-codecs  # すべて有効化
```

## クイックスタート

```bash
# 画像情報を表示
panimg info photo.jpg
panimg info photo.jpg --format json --fields width,height

# フォーマット変換
panimg convert photo.png -o photo.webp --quality 80

# リサイズ
panimg resize photo.jpg --width 800 -o thumbnail.jpg

# クロップ・回転・反転
panimg crop photo.jpg --x 100 --y 100 --width 400 --height 300 -o cropped.jpg
panimg rotate photo.jpg --angle 90 -o rotated.jpg
panimg flip photo.jpg --direction horizontal -o flipped.jpg

# 色調整
panimg brightness photo.jpg --value 20 -o brighter.jpg
panimg saturate photo.jpg --factor 1.5 -o vivid.jpg
panimg sepia photo.jpg -o vintage.jpg

# フィルター
panimg blur photo.jpg --sigma 3.0 -o blurred.jpg
panimg sharpen photo.jpg --sigma 1.5 -o sharp.jpg
panimg edge-detect photo.jpg -o edges.jpg

# カラーエフェクト
panimg tint photo.jpg --color red --strength 0.3 -o warm.jpg
panimg posterize photo.jpg --levels 4 -o poster.jpg

# 描画
panimg draw photo.jpg --shape rect --x 10 --y 10 --width 100 --height 50 --color red -o annotated.jpg

# テキスト描画
panimg text photo.jpg --content "Hello World" --size 48 --color white --position center -o titled.jpg
panimg text photo.jpg --content "© 2026" --size 16 --color '#FFFFFF80' --position bottom-right -o stamped.jpg

# 合成
panimg overlay base.jpg --layer watermark.png --position bottom-right --opacity 0.5 -o watermarked.jpg

# GIF アニメーション
panimg frames animation.gif --output-dir ./frames
panimg animate 'frames/*.png' -o animation.gif --delay 100
panimg gif-speed animation.gif -o fast.gif --speed 2.0

# 画像比較
panimg diff before.png after.png -o diff.png

# パイプライン（マルチステップ）
panimg pipeline photo.jpg -o result.jpg --steps "resize --width 800 | blur --sigma 1.5 | grayscale"

# バッチ処理
panimg batch convert 'photos/*.png' --output-dir ./webp --to webp --quality 80
panimg batch resize 'photos/*.jpg' --output-dir ./thumbs --width 200

# ドライラン＆スキーマ
panimg resize photo.jpg --width 800 -o out.jpg --dry-run --format json
panimg resize --schema
panimg --capabilities --format json
```

## コマンド一覧

### 基本操作

| コマンド | 説明 |
|----------|------|
| `info` | 画像のメタデータとプロパティを表示 |
| `convert` | 画像フォーマットを変換 |
| `resize` | 複数のフィットモードで画像をリサイズ |
| `crop` | 矩形領域をクロップ |
| `rotate` | 90、180、270 度回転 |
| `flip` | 水平または垂直に反転 |
| `auto-orient` | EXIF の方向タグに基づいて自動回転 |

### 色調整

| コマンド | 説明 |
|----------|------|
| `brightness` | 明るさを調整 |
| `contrast` | コントラストを調整 |
| `hue-rotate` | 色相を回転 |
| `saturate` | 彩度を調整 |
| `grayscale` | グレースケールに変換 |
| `invert` | 色を反転 |
| `sepia` | セピアトーンを適用 |
| `tint` | 指定色でティントを適用 |
| `posterize` | チャンネルあたりの色数を削減 |

### フィルター

| コマンド | 説明 |
|----------|------|
| `blur` | ガウシアンブラー |
| `sharpen` | アンシャープマスクでシャープ化 |
| `edge-detect` | エッジ検出（Laplacian カーネル） |
| `emboss` | エンボス効果 |

### 描画と合成

| コマンド | 説明 |
|----------|------|
| `draw` | 図形を描画（矩形、円、線） |
| `text` | テキストを描画（内蔵またはカスタムフォント対応） |
| `overlay` | 画像を重ね合わせて合成 |
| `trim` | 余白や類似色の境界を自動クロップ |
| `diff` | 2 枚の画像を比較し差分を可視化 |

### アニメーション

| コマンド | 説明 |
|----------|------|
| `frames` | アニメーション GIF から個別フレームを抽出 |
| `animate` | 複数の画像をアニメーション GIF に組み立て |
| `gif-speed` | アニメーションの再生速度を変更 |

### ワークフロー

| コマンド | 説明 |
|----------|------|
| `pipeline` | 単一の読み書きで複数操作を実行 |
| `batch` | glob パターンで複数ファイルを一括処理 |

## 対応フォーマット

| フォーマット | デコード | エンコード | 備考 |
|--------------|----------|------------|------|
| JPEG | Yes | Yes | 品質 1-100 |
| PNG | Yes | Yes | |
| WebP | Yes | Yes | 品質 1-100 |
| GIF | Yes | Yes | アニメーション GIF 対応 |
| BMP | Yes | Yes | |
| TIFF | Yes | Yes | |
| QOI | Yes | Yes | |
| AVIF | Yes | Yes | `avif` feature が必要 |
| JPEG XL | Yes | No | `jxl` feature が必要、デコードのみ |
| SVG | Yes | No | `svg` feature が必要、ラスタライズのみ |

## AI エージェント連携

panimg は AI エージェントや自動化スクリプトとのシームレスな連携を想定して設計されています：

```bash
# プログラムから機能を探索
panimg --capabilities --format json

# 任意のコマンドのパラメータスキーマを取得
panimg resize --schema

# 副作用なしで操作をプレビュー
panimg resize photo.jpg --width 800 -o out.jpg --dry-run --format json

# すべての出力を構造化 JSON で取得可能
panimg info photo.jpg --format json --fields width,height,format
```

## 終了コード

| コード | 意味 |
|--------|------|
| 0 | 成功 |
| 1 | 一般エラー |
| 2 | 入力ファイルエラー（ファイル未検出、権限不足、デコード失敗） |
| 3 | 出力の問題（書き込み失敗、ファイル既存） |
| 4 | 未対応フォーマット |
| 5 | 引数エラー |

## エラー出力

エラーメッセージは構造化されており、実行可能な提案が含まれます：

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

## ライセンス

本プロジェクトは以下のいずれかのライセンスの下で提供されます：

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

お好みでお選びください。
