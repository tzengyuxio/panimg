# コマンドリファレンス

panimg は画像処理、色操作、フィルター、描画、アニメーション、最適化、ワークフロー自動化をカバーする豊富なコマンドセットを提供します。すべてのコマンドは一貫した `panimg <command> <input> [options]` 構文に従います。

## グローバルオプション

すべてのコマンドで以下のフラグが使用できます：

| フラグ | 説明 |
|--------|------|
| `-o`, `--output` | 出力ファイルパス |
| `--format` | 出力形式：`text`（デフォルト）または `json` |
| `--dry-run` | ファイルを書き込まずに操作をプレビュー |
| `--schema` | コマンドのパラメータ定義を JSON として出力 |

さらに、以下のグローバルフラグも利用できます：

```bash
panimg --capabilities              # サポートされているすべてのコマンド、フォーマット、機能を一覧表示
panimg --capabilities --format json  # 同上、構造化 JSON として出力
```

---

## 基本操作

### `info`

画像のメタデータとプロパティを表示します。

```bash
panimg info photo.jpg
panimg info photo.jpg --format json
panimg info photo.jpg --format json --fields width,height,format
```

| オプション | 説明 |
|------------|------|
| `--format` | 出力形式：`text` または `json` |
| `--fields` | 含めるフィールドのカンマ区切りリスト（JSON モード） |

### `convert`

画像をフォーマット変換します。出力フォーマットはファイル拡張子から推測されます。

```bash
panimg convert photo.png -o photo.webp
panimg convert photo.png -o photo.webp --quality 80
```

| オプション | 説明 |
|------------|------|
| `-o`, `--output` | 出力ファイルパス（必須） |
| `--quality` | 品質レベル 1-100（JPEG、WebP、AVIF 用） |

### `resize`

複数のフィットモードで画像をリサイズします。

```bash
panimg resize photo.jpg --width 800 -o thumbnail.jpg
panimg resize photo.jpg --width 800 --height 600 --fit cover -o thumb.jpg
```

| オプション | 説明 |
|------------|------|
| `--width` | ターゲット幅（ピクセル） |
| `--height` | ターゲット高さ（ピクセル） |
| `--fit` | フィットモード：`contain`（デフォルト）、`cover`、`fill`、`inside`、`outside` |
| `-o`, `--output` | 出力ファイルパス |

### `crop`

画像から矩形領域をクロップします。

```bash
panimg crop photo.jpg --x 100 --y 100 --width 400 --height 300 -o cropped.jpg
```

| オプション | 説明 |
|------------|------|
| `--x` | 左オフセット（ピクセル） |
| `--y` | 上オフセット（ピクセル） |
| `--width` | クロップ幅（ピクセル） |
| `--height` | クロップ高さ（ピクセル） |
| `-o`, `--output` | 出力ファイルパス |

### `rotate`

90、180、270 度で回転します。

```bash
panimg rotate photo.jpg --angle 90 -o rotated.jpg
```

| オプション | 説明 |
|------------|------|
| `--angle` | 回転角度：`90`、`180`、`270` |
| `-o`, `--output` | 出力ファイルパス |

### `flip`

水平または垂直に反転します。

```bash
panimg flip photo.jpg --direction horizontal -o flipped.jpg
```

| オプション | 説明 |
|------------|------|
| `--direction` | 反転方向：`horizontal` または `vertical` |
| `-o`, `--output` | 出力ファイルパス |

### `auto-orient`

EXIF の方向タグに基づいて自動回転し、タグを削除します。

```bash
panimg auto-orient photo.jpg -o oriented.jpg
```

---

## 色調整

### `brightness`

画像の明るさを調整します。

```bash
panimg brightness photo.jpg --value 20 -o brighter.jpg
panimg brightness photo.jpg --value -10 -o darker.jpg
```

| オプション | 説明 |
|------------|------|
| `--value` | 明るさの調整値（正 = より明るく、負 = より暗く） |

### `contrast`

画像のコントラストを調整します。

```bash
panimg contrast photo.jpg --value 1.5 -o enhanced.jpg
```

| オプション | 説明 |
|------------|------|
| `--value` | コントラスト倍率（1.0 = 変化なし、>1 = コントラスト増加） |

### `hue-rotate`

画像の色相を回転します。

```bash
panimg hue-rotate photo.jpg --degrees 90 -o shifted.jpg
```

| オプション | 説明 |
|------------|------|
| `--degrees` | 色相回転の角度 |

### `saturate`

彩度を調整します。

```bash
panimg saturate photo.jpg --factor 1.5 -o vivid.jpg
panimg saturate photo.jpg --factor 0.5 -o muted.jpg
```

| オプション | 説明 |
|------------|------|
| `--factor` | 彩度倍率（1.0 = 変化なし、0 = グレースケール、>1 = より鮮やか） |

### `grayscale`

グレースケールに変換します。

```bash
panimg grayscale photo.jpg -o bw.jpg
```

### `invert`

すべての色を反転（ネガ）します。

```bash
panimg invert photo.jpg -o inverted.jpg
```

### `sepia`

セピアトーン効果を適用します。

```bash
panimg sepia photo.jpg -o vintage.jpg
```

### `tint`

指定した色で画像をティントします。

```bash
panimg tint photo.jpg --color red --strength 0.3 -o warm.jpg
panimg tint photo.jpg --color '#FF6600' --strength 0.5 -o orange.jpg
```

| オプション | 説明 |
|------------|------|
| `--color` | ティント色（色名または 16 進数） |
| `--strength` | ティント強度（0.0 〜 1.0） |

### `posterize`

チャンネルあたりの色レベル数を削減します。

```bash
panimg posterize photo.jpg --levels 4 -o poster.jpg
```

| オプション | 説明 |
|------------|------|
| `--levels` | チャンネルあたりの色レベル数 |

---

## フィルター

### `blur`

ガウシアンブラーを適用します。

```bash
panimg blur photo.jpg --sigma 3.0 -o blurred.jpg
```

| オプション | 説明 |
|------------|------|
| `--sigma` | ブラー半径（大きいほどぼかしが強い） |

### `sharpen`

アンシャープマスクでシャープ化します。

```bash
panimg sharpen photo.jpg --sigma 1.5 -o sharp.jpg
```

| オプション | 説明 |
|------------|------|
| `--sigma` | シャープ化の強度 |

### `edge-detect`

Laplacian カーネルを使用してエッジ検出を行います。

```bash
panimg edge-detect photo.jpg -o edges.jpg
```

### `emboss`

エンボス効果を適用します。

```bash
panimg emboss photo.jpg -o embossed.jpg
```

---

## 描画とコンポジット

### `draw`

画像上に図形を描画します。

```bash
panimg draw photo.jpg --shape rect --x 10 --y 10 --width 100 --height 50 --color red -o annotated.jpg
panimg draw photo.jpg --shape circle --cx 200 --cy 200 --radius 50 --color blue --fill -o marked.jpg
panimg draw photo.jpg --shape line --x1 0 --y1 0 --x2 100 --y2 100 --color white -o lined.jpg
```

| オプション | 説明 |
|------------|------|
| `--shape` | 図形の種類：`rect`、`circle`、`line` |
| `--x`, `--y` | 位置（rect） |
| `--cx`, `--cy` | 中心位置（circle） |
| `--x1`, `--y1`, `--x2`, `--y2` | 始点/終点（line） |
| `--width`, `--height` | サイズ（rect） |
| `--radius` | 半径（circle） |
| `--color` | 図形の色（色名または 16 進数） |
| `--fill` | 図形を塗りつぶす（フラグ） |

### `text`

内蔵フォントまたはカスタムフォントでテキストを描画します。

```bash
panimg text photo.jpg --content "Hello World" --size 48 --color white --position center -o titled.jpg
panimg text photo.jpg --content "© 2026" --size 16 --color '#FFFFFF80' --position bottom-right -o stamped.jpg
```

| オプション | 説明 |
|------------|------|
| `--content` | 描画するテキスト文字列 |
| `--size` | フォントサイズ（ピクセル） |
| `--color` | テキスト色（色名、16 進数、アルファ付き 16 進数） |
| `--position` | 配置：`center`、`top-left`、`top-right`、`bottom-left`、`bottom-right` |
| `--font` | カスタムフォントファイルのパス（オプション） |

### `overlay`

画像を重ね合わせて合成します。

```bash
panimg overlay base.jpg --layer watermark.png --position bottom-right --opacity 0.5 -o watermarked.jpg
```

| オプション | 説明 |
|------------|------|
| `--layer` | オーバーレイ画像のパス |
| `--position` | 配置位置 |
| `--opacity` | オーバーレイの不透明度（0.0 〜 1.0） |

### `trim`

余白や類似色の境界を自動クロップします。

```bash
panimg trim photo.jpg -o trimmed.jpg
```

### `diff`

2 枚の画像を比較し差分を可視化します。

```bash
panimg diff before.png after.png -o diff.png
```

---

## アニメーション

### `frames`

アニメーション GIF から個別フレームを抽出します。

```bash
panimg frames animation.gif --output-dir ./frames
```

| オプション | 説明 |
|------------|------|
| `--output-dir` | 抽出したフレームの保存先ディレクトリ |

### `animate`

複数の画像をアニメーション GIF に組み立てます。

```bash
panimg animate 'frames/*.png' -o animation.gif --delay 100
```

| オプション | 説明 |
|------------|------|
| `--delay` | フレーム間の遅延時間（ミリ秒） |
| `-o`, `--output` | 出力 GIF ファイルパス |

### `gif-speed`

アニメーションの再生速度を変更します。

```bash
panimg gif-speed animation.gif -o fast.gif --speed 2.0
panimg gif-speed animation.gif -o slow.gif --speed 0.5
```

| オプション | 説明 |
|------------|------|
| `--speed` | 速度倍率（2.0 = 2 倍速、0.5 = 半速） |

---

## 最適化

### `tiny`

スマート画像圧縮——フォーマットに応じて最適な圧縮戦略を自動選択します（TinyPNG 風）。

- **PNG**：非可逆量子化（imagequant）+ 可逆最適化（oxipng）
- **JPEG**：品質制御付き再エンコード（デフォルト品質 75）
- **WebP**：品質制御付きエンコード（デフォルト品質 75）
- **AVIF**：品質制御付きエンコード（デフォルト品質 68、`avif` feature が必要）

```bash
panimg tiny photo.png                           # → photo_tiny.png
panimg tiny photo.png -o compressed.png         # 出力パスを指定
panimg tiny photo.jpg --quality 60              # カスタム品質
panimg tiny icon.png --lossless                 # PNG：可逆最適化のみ
panimg tiny photo.png --max-colors 128          # PNG：パレット色数を制限
panimg tiny photo.png --strip                   # メタデータを除去
panimg batch tiny 'photos/*.png' --output-dir compressed/  # バッチモード
```

| オプション | 説明 |
|------------|------|
| `-o`, `--output` | 出力ファイルパス（デフォルト：`{stem}_tiny.{ext}`） |
| `--quality` | 圧縮品質 1-100（JPEG、WebP、AVIF） |
| `--max-colors` | PNG：量子化の最大パレット色数（2-256、デフォルト 256） |
| `--lossless` | PNG：量子化をスキップし、可逆最適化のみ実行 |
| `--strip` | 出力からメタデータを除去 |

---

## ワークフロー

### `pipeline`

単一の読み書きパスで複数の操作を実行します。

```bash
panimg pipeline photo.jpg -o result.jpg --steps "resize --width 800 | blur --sigma 1.5 | grayscale"
panimg pipeline photo.jpg -o result.jpg --steps "brightness --value 10 | contrast --value 1.2 | sharpen --sigma 1.0"
```

| オプション | 説明 |
|------------|------|
| `--steps` | パイプ区切りの操作リスト |
| `-o`, `--output` | 出力ファイルパス |

### `batch`

glob パターンと並列実行で複数ファイルを一括処理します。

```bash
panimg batch convert 'photos/*.png' --output-dir ./webp --to webp --quality 80
panimg batch resize 'photos/*.jpg' --output-dir ./thumbs --width 200
```

| オプション | 説明 |
|------------|------|
| `--output-dir` | 出力ファイルのディレクトリ |
| `--to` | ターゲットフォーマット（`batch convert` 用） |
| その他のオプション | コマンド固有のオプションがそのまま渡されます |
