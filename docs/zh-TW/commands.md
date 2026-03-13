# 命令參考

panimg 提供豐富的命令集，涵蓋圖片處理、色彩調整、濾鏡、繪圖、動畫、最佳化與工作流程自動化。所有命令遵循一致的 `panimg <command> <input> [options]` 語法。

## 全域選項

每個命令都支援以下旗標：

| 旗標 | 說明 |
|------|------|
| `-o`, `--output` | 輸出檔案路徑 |
| `--format` | 輸出格式：`text`（預設）或 `json` |
| `--dry-run` | 預覽操作，不寫入任何檔案 |
| `--schema` | 以 JSON 格式輸出該命令的參數定義 |

此外，還有以下全域旗標可用：

```bash
panimg --capabilities              # 列出所有支援的命令、格式與功能
panimg --capabilities --format json  # 同上，以結構化 JSON 輸出
```

---

## 基本操作

### `info`

顯示圖片 metadata 與屬性。

```bash
panimg info photo.jpg
panimg info photo.jpg --format json
panimg info photo.jpg --format json --fields width,height,format
```

| 選項 | 說明 |
|------|------|
| `--format` | 輸出格式：`text` 或 `json` |
| `--fields` | 以逗號分隔的欄位清單（JSON 模式）。啟用 `icc` feature 時可包含 `icc_profile` |

### `convert`

圖片格式轉換。輸出格式由副檔名自動推斷。

```bash
panimg convert photo.png -o photo.webp
panimg convert photo.png -o photo.webp --quality 80
panimg convert photo.jpg --convert-profile display-p3 -o photo-p3.jpg
panimg convert document.pdf -o page1.png --dpi 300
panimg convert photo.heic -o photo.jpg
```

| 選項 | 說明 |
|------|------|
| `-o`, `--output` | 輸出檔案路徑（必填） |
| `--quality` | 品質等級 1-100（適用於 JPEG、WebP、AVIF） |
| `--dpi` | PDF 輸入的光柵化 DPI（預設：150） |
| `--convert-profile` | 轉換至目標色彩空間：`srgb`、`adobe-rgb`、`display-p3`（需啟用 `icc` feature） |
| `--strip` | 移除輸出檔案的 metadata |

### `resize`

縮放圖片，支援多種適配模式。

```bash
panimg resize photo.jpg --width 800 -o thumbnail.jpg
panimg resize photo.jpg --width 800 --height 600 --fit cover -o thumb.jpg
```

| 選項 | 說明 |
|------|------|
| `--width` | 目標寬度（像素） |
| `--height` | 目標高度（像素） |
| `--fit` | 適配模式：`contain`（預設）、`cover`、`fill`、`inside`、`outside` |
| `-o`, `--output` | 輸出檔案路徑 |

### `crop`

從圖片裁切矩形區域。

```bash
panimg crop photo.jpg --x 100 --y 100 --width 400 --height 300 -o cropped.jpg
```

| 選項 | 說明 |
|------|------|
| `--x` | 左側偏移量（像素） |
| `--y` | 頂部偏移量（像素） |
| `--width` | 裁切寬度（像素） |
| `--height` | 裁切高度（像素） |
| `-o`, `--output` | 輸出檔案路徑 |

### `rotate`

任意角度旋轉。90、180、270 度使用快速無損路徑；任意角度（如 45、30.5）使用雙線性插值。

```bash
panimg rotate photo.jpg --angle 90 -o rotated.jpg
panimg rotate photo.jpg --angle 45 --background white -o rotated.jpg
```

| 選項 | 說明 |
|------|------|
| `--angle` | 旋轉角度：`90`、`180`、`270`、`left`、`right`，或任意數值（如 `45`、`30.5`） |
| `--background` | 任意角度旋轉時的背景填充色。支援 hex（`#FF0000`）、RGB（`255,0,0`）或命名色彩（`white`、`transparent` 等）。預設：支援透明的格式用 `transparent`，JPEG/BMP 用 `white` |
| `-o`, `--output` | 輸出檔案路徑 |

### `flip`

水平或垂直翻轉。

```bash
panimg flip photo.jpg --direction horizontal -o flipped.jpg
```

| 選項 | 說明 |
|------|------|
| `--direction` | 翻轉方向：`horizontal`（水平）或 `vertical`（垂直） |
| `-o`, `--output` | 輸出檔案路徑 |

### `auto-orient`

依據 EXIF 方向標籤自動旋轉，然後移除該標籤。

```bash
panimg auto-orient photo.jpg -o oriented.jpg
```

---

## 色彩調整

### `brightness`

調整圖片亮度。

```bash
panimg brightness photo.jpg --value 20 -o brighter.jpg
panimg brightness photo.jpg --value -10 -o darker.jpg
```

| 選項 | 說明 |
|------|------|
| `--value` | 亮度調整值（正值 = 更亮，負值 = 更暗） |

### `contrast`

調整圖片對比度。

```bash
panimg contrast photo.jpg --value 1.5 -o enhanced.jpg
```

| 選項 | 說明 |
|------|------|
| `--value` | 對比度倍率（1.0 = 不變，>1 = 增強對比） |

### `hue-rotate`

色相旋轉。

```bash
panimg hue-rotate photo.jpg --degrees 90 -o shifted.jpg
```

| 選項 | 說明 |
|------|------|
| `--degrees` | 色相旋轉角度 |

### `saturate`

調整色彩飽和度。

```bash
panimg saturate photo.jpg --factor 1.5 -o vivid.jpg
panimg saturate photo.jpg --factor 0.5 -o muted.jpg
```

| 選項 | 說明 |
|------|------|
| `--factor` | 飽和度倍率（1.0 = 不變，0 = 灰階，>1 = 更飽和） |

### `grayscale`

轉為灰階。

```bash
panimg grayscale photo.jpg -o bw.jpg
```

### `invert`

反轉所有色彩。

```bash
panimg invert photo.jpg -o inverted.jpg
```

### `sepia`

套用復古棕褐色調效果。

```bash
panimg sepia photo.jpg -o vintage.jpg
```

### `tint`

以指定色彩為圖片著色。

```bash
panimg tint photo.jpg --color red --strength 0.3 -o warm.jpg
panimg tint photo.jpg --color '#FF6600' --strength 0.5 -o orange.jpg
```

| 選項 | 說明 |
|------|------|
| `--color` | 著色色彩（色彩名稱或十六進位值） |
| `--strength` | 著色強度，範圍 0.0 到 1.0 |

### `posterize`

減少每通道色階數。

```bash
panimg posterize photo.jpg --levels 4 -o poster.jpg
```

| 選項 | 說明 |
|------|------|
| `--levels` | 每通道色階數 |

---

## 濾鏡

### `blur`

套用高斯模糊。

```bash
panimg blur photo.jpg --sigma 3.0 -o blurred.jpg
```

| 選項 | 說明 |
|------|------|
| `--sigma` | 模糊半徑（數值越大越模糊） |

### `sharpen`

使用非銳化遮罩進行銳化。

```bash
panimg sharpen photo.jpg --sigma 1.5 -o sharp.jpg
```

| 選項 | 說明 |
|------|------|
| `--sigma` | 銳化強度 |

### `edge-detect`

使用 Laplacian 核進行邊緣偵測。

```bash
panimg edge-detect photo.jpg -o edges.jpg
```

### `emboss`

套用浮雕效果。

```bash
panimg emboss photo.jpg -o embossed.jpg
```

### `tilt-shift`

套用移軸攝影（微縮模型）效果。保持水平帶狀區域對焦，同時漸進模糊上下區域。

```bash
panimg tilt-shift photo.jpg -o miniature.jpg
panimg tilt-shift photo.jpg --sigma 12 --focus-position 0.4 --focus-width 0.2 --saturation 1.3 -o miniature.jpg
```

| 選項 | 說明 |
|------|------|
| `--sigma` | 失焦區域模糊強度（預設：8.0） |
| `--focus-position` | 對焦帶垂直中心，0=頂部，1=底部（預設：0.5） |
| `--focus-width` | 對焦帶高度佔圖片高度比例（預設：0.15） |
| `--transition` | 過渡區域寬度佔圖片高度比例（預設：0.2） |
| `--saturation` | 飽和度倍數，>1 增強微縮感（預設：1.0） |

---

## 繪圖與合成

### `draw`

在圖片上繪製圖形。

```bash
panimg draw photo.jpg --shape rect --x 10 --y 10 --width 100 --height 50 --color red -o annotated.jpg
panimg draw photo.jpg --shape circle --cx 200 --cy 200 --radius 50 --color blue --fill -o marked.jpg
panimg draw photo.jpg --shape line --x1 0 --y1 0 --x2 100 --y2 100 --color white -o lined.jpg
```

| 選項 | 說明 |
|------|------|
| `--shape` | 圖形類型：`rect`（矩形）、`circle`（圓形）、`line`（線段） |
| `--x`, `--y` | 位置（矩形） |
| `--cx`, `--cy` | 中心位置（圓形） |
| `--x1`, `--y1`, `--x2`, `--y2` | 起點/終點（線段） |
| `--width`, `--height` | 尺寸（矩形） |
| `--radius` | 半徑（圓形） |
| `--color` | 圖形色彩（色彩名稱或十六進位值） |
| `--fill` | 填滿圖形（旗標） |

### `text`

繪製文字，支援內嵌或自訂字型。

```bash
panimg text photo.jpg --content "Hello World" --size 48 --color white --position center -o titled.jpg
panimg text photo.jpg --content "© 2026" --size 16 --color '#FFFFFF80' --position bottom-right -o stamped.jpg
```

| 選項 | 說明 |
|------|------|
| `--content` | 要渲染的文字字串 |
| `--size` | 字型大小（像素） |
| `--color` | 文字色彩（色彩名稱、十六進位值，或含透明度的十六進位值） |
| `--position` | 位置：`center`、`top-left`、`top-right`、`bottom-left`、`bottom-right` |
| `--font` | 自訂字型檔案路徑（選填） |

### `overlay`

疊加合成圖片。

```bash
panimg overlay base.jpg --layer watermark.png --position bottom-right --opacity 0.5 -o watermarked.jpg
```

| 選項 | 說明 |
|------|------|
| `--layer` | 疊加圖層的圖片路徑 |
| `--position` | 放置位置 |
| `--opacity` | 疊加圖層透明度，範圍 0.0 到 1.0 |

### `trim`

自動裁切空白或相似色邊框。

```bash
panimg trim photo.jpg -o trimmed.jpg
```

### `diff`

比較兩張圖片並視覺化差異。

```bash
panimg diff before.png after.png -o diff.png
```

---

## 動畫

### `frames`

從動態 GIF 提取個別幀。

```bash
panimg frames animation.gif --output-dir ./frames
```

| 選項 | 說明 |
|------|------|
| `--output-dir` | 儲存提取幀的目錄 |

### `animate`

將多張圖片組合成動態 GIF。

```bash
panimg animate 'frames/*.png' -o animation.gif --delay 100
```

| 選項 | 說明 |
|------|------|
| `--delay` | 幀間延遲時間（毫秒） |
| `-o`, `--output` | 輸出 GIF 檔案路徑 |

### `gif-speed`

調整動畫播放速度。

```bash
panimg gif-speed animation.gif -o fast.gif --speed 2.0
panimg gif-speed animation.gif -o slow.gif --speed 0.5
```

| 選項 | 說明 |
|------|------|
| `--speed` | 速度倍率（2.0 = 加速兩倍，0.5 = 減速一半） |

---

## 最佳化

### `tiny`

智慧圖片壓縮——根據格式自動選擇最佳壓縮策略（類似 TinyPNG）。

- **PNG**：有損量化（imagequant）+ 無損最佳化（oxipng）
- **JPEG**：品質控制重新編碼（預設品質 75）
- **WebP**：品質控制編碼（預設品質 75）
- **AVIF**：品質控制編碼（預設品質 68，需要 `avif` feature）

```bash
panimg tiny photo.png                           # → photo_tiny.png
panimg tiny photo.png -o compressed.png         # 指定輸出路徑
panimg tiny photo.jpg --quality 60              # 自訂品質
panimg tiny icon.png --lossless                 # PNG：僅無損最佳化
panimg tiny photo.png --max-colors 128          # PNG：限制調色盤色數
panimg tiny photo.png --strip                   # 移除 metadata
panimg batch tiny 'photos/*.png' --output-dir compressed/  # 批次模式
```

| 選項 | 說明 |
|------|------|
| `-o`, `--output` | 輸出檔案路徑（預設：`{stem}_tiny.{ext}`） |
| `--quality` | 壓縮品質 1-100（JPEG、WebP、AVIF） |
| `--max-colors` | PNG：量化最大調色盤色數（2-256，預設 256） |
| `--lossless` | PNG：跳過量化，僅進行無損最佳化 |
| `--strip` | 從輸出移除 metadata |

---

## 工作流程

### `pipeline`

在單次讀寫中執行多個操作。

```bash
panimg pipeline photo.jpg -o result.jpg --steps "resize --width 800 | blur --sigma 1.5 | grayscale"
panimg pipeline photo.jpg -o result.jpg --steps "brightness --value 10 | contrast --value 1.2 | sharpen --sigma 1.0"
```

| 選項 | 說明 |
|------|------|
| `--steps` | 以管線符號分隔的操作清單 |
| `-o`, `--output` | 輸出檔案路徑 |

### `batch`

以 glob 模式匹配多個檔案並平行處理。

```bash
panimg batch convert 'photos/*.png' --output-dir ./webp --to webp --quality 80
panimg batch resize 'photos/*.jpg' --output-dir ./thumbs --width 200
```

| 選項 | 說明 |
|------|------|
| `--output-dir` | 輸出檔案目錄 |
| `--to` | 目標格式（用於 `batch convert`） |
| 其他選項 | 各命令專屬選項會被傳遞 |
