# panimg

[![CI](https://github.com/tzengyuxio/panimg/actions/workflows/ci.yml/badge.svg)](https://github.com/tzengyuxio/panimg/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-2021_edition-orange.svg)](https://www.rust-lang.org/)

[English](README.md) | [日本語](README.ja.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md)

画像処理のスイスアーミーナイフ——人間にも AI エージェントにも。

## 特徴

- **豊富なコマンドセット**でフォーマット変換、リサイズ、クロップ、回転、色調整、フィルター、テキスト、合成、アニメーション、圧縮などをカバー
- **パイプラインエンジン**——単一の読み書きで複数操作を連鎖実行
- **バッチ処理**——glob パターンによるマルチファイル処理を並列実行
- **AI エージェント対応**——構造化 JSON 出力、`--dry-run`、`--schema`、`--capabilities` でプログラム的に利用可能
- **高速かつ安全**——Rust で構築、SIMD アクセラレーテッドリサイズ、コアパイプラインに unsafe コードなし（[ベンチマーク](benches/results/REPORT.md)）

## インストール

### Homebrew (macOS / Linux)

```bash
brew install tzengyuxio/tap/panimg
```

### Cargo

```bash
cargo install panimg-cli
```

### ソースからビルド

```bash
git clone https://github.com/tzengyuxio/panimg.git
cd panimg
cargo build --release
```

オプションコーデック（AVIF、JPEG XL、SVG、PDF、HEIC）については[対応フォーマット](docs/ja/formats.md)を参照してください。

## クイックスタート

```bash
# 画像情報を取得
panimg info photo.jpg --format json

# フォーマット変換
panimg convert photo.png -o photo.webp --quality 80

# カラースペース変換（icc feature 必要）
panimg convert photo.jpg --convert-profile display-p3 -o photo-p3.jpg

# PDF 最初のページを PNG に変換（pdf feature 必要）
panimg convert document.pdf -o page1.png --dpi 300

# HEIC を JPEG に変換（heic feature 必要、macOS のみ）
panimg convert photo.heic -o photo.jpg

# フィットモード指定リサイズ
panimg resize photo.jpg --width 800 --height 600 --fit cover -o thumb.jpg

# 色調整
panimg saturate photo.jpg --factor 1.5 -o vivid.jpg

# フィルター適用
panimg blur photo.jpg --sigma 3.0 -o blurred.jpg

# 透かしテキストを追加
panimg text photo.jpg --content "© 2026" --size 24 --color white --position bottom-right -o stamped.jpg

# スマート圧縮
panimg tiny photo.png -o compressed.png

# マルチステップパイプライン
panimg pipeline photo.jpg -o result.jpg --steps "resize --width 800 | blur --sigma 1.5 | grayscale"

# バッチ処理
panimg batch convert 'photos/*.png' --output-dir ./webp --to webp --quality 80
```

詳細な使い方と例については[コマンドリファレンス](docs/ja/commands.md)を参照してください。

## コマンド一覧

### 情報と変換

| コマンド | 説明 |
|----------|------|
| `info` | 画像のメタデータとプロパティを表示 |
| `convert` | 画像フォーマットを変換 |
| `tiny` | スマート圧縮（TinyPNG 風） |

### 幾何変換

| コマンド | 説明 |
|----------|------|
| `resize` | 複数のフィットモードで画像をリサイズ |
| `crop` | 矩形領域をクロップ |
| `smart-crop` | 画像内容に基づく最適トリミング領域の自動選択 |
| `trim` | 余白や類似色の境界を自動クロップ |
| `rotate` | 任意の角度で回転（90/180/270 はロスレス高速パスを使用） |
| `flip` | 水平または垂直に反転 |
| `auto-orient` | EXIF の方向タグに基づいて自動回転 |

### 色彩とトーン

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

### フィルターとエフェクト

| コマンド | 説明 |
|----------|------|
| `blur` | ガウシアンブラー |
| `sharpen` | アンシャープマスクでシャープ化 |
| `edge-detect` | エッジ検出（Laplacian カーネル） |
| `emboss` | エンボス効果 |
| `tilt-shift` | ミニチュア/ジオラマ風チルトシフト効果 |

### 合成と描画

| コマンド | 説明 |
|----------|------|
| `draw` | 図形を描画（矩形、円、線） |
| `text` | テキストを描画（内蔵またはカスタムフォント対応） |
| `overlay` | 画像を重ね合わせて合成 |

### 比較とアニメーション

| コマンド | 説明 |
|----------|------|
| `diff` | 2 枚の画像を比較し差分を可視化 |
| `frames` | アニメーション GIF から個別フレームを抽出 |
| `animate` | 複数の画像をアニメーション GIF に組み立て |
| `gif-speed` | アニメーションの再生速度を変更 |

### 自動化

| コマンド | 説明 |
|----------|------|
| `pipeline` | 単一の読み書きで複数操作を実行 |
| `batch` | glob パターンで複数ファイルを一括処理 |

詳細な使い方と例については[コマンドリファレンス](docs/ja/commands.md)を参照してください。

## AI エージェント連携

panimg はプログラム的な探索と構造化出力に対応し、AI エージェントや自動化スクリプトに最適です：

```bash
panimg --capabilities --format json   # すべてのコマンドとフォーマットを探索
panimg resize --schema                # パラメータ定義を JSON で取得
panimg resize photo.jpg --width 800 -o out.jpg --dry-run --format json  # 副作用なしでプレビュー
```

終了コード、エラー形式、連携パターンについては [AI 連携ガイド](docs/ja/ai-integration.md)を参照してください。

## ドキュメント

- [コマンドリファレンス](docs/ja/commands.md) — 全コマンドの詳細な使い方と例
- [対応フォーマット](docs/ja/formats.md) — フォーマット対応表とオプションコーデック
- [AI 連携ガイド](docs/ja/ai-integration.md) — 構造化出力、スキーマ、ドライラン、終了コード

## ライセンス

本プロジェクトは以下のいずれかのライセンスの下で提供されます：

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

お好みでお選びください。
