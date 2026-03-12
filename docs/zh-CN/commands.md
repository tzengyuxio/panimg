# 命令参考

panimg 提供 31 个命令，涵盖图片处理、色彩调整、滤镜、绘制、动画和工作流自动化。所有命令遵循一致的 `panimg <command> <input> [options]` 语法。

## 全局选项

每个命令都支持以下标志：

| 标志 | 说明 |
|------|------|
| `-o`, `--output` | 输出文件路径 |
| `--format` | 输出格式：`text`（默认）或 `json` |
| `--dry-run` | 预览操作而不写入任何文件 |
| `--schema` | 以 JSON 格式打印命令的参数定义 |

此外，还可使用以下全局标志：

```bash
panimg --capabilities              # 列出所有支持的命令、格式和功能
panimg --capabilities --format json  # 同上，以结构化 JSON 输出
```

---

## 基本操作

### `info`

显示图片 metadata 与属性。

```bash
panimg info photo.jpg
panimg info photo.jpg --format json
panimg info photo.jpg --format json --fields width,height,format
```

| 选项 | 说明 |
|------|------|
| `--format` | 输出格式：`text` 或 `json` |
| `--fields` | 以逗号分隔的字段列表（JSON 模式） |

### `convert`

图片格式转换。输出格式根据文件扩展名自动推断。

```bash
panimg convert photo.png -o photo.webp
panimg convert photo.png -o photo.webp --quality 80
```

| 选项 | 说明 |
|------|------|
| `-o`, `--output` | 输出文件路径（必填） |
| `--quality` | 质量等级 1-100（适用于 JPEG、WebP、AVIF） |

### `resize`

缩放图片，支持多种适配模式。

```bash
panimg resize photo.jpg --width 800 -o thumbnail.jpg
panimg resize photo.jpg --width 800 --height 600 --fit cover -o thumb.jpg
```

| 选项 | 说明 |
|------|------|
| `--width` | 目标宽度（像素） |
| `--height` | 目标高度（像素） |
| `--fit` | 适配模式：`contain`（默认）、`cover`、`fill`、`inside`、`outside` |
| `-o`, `--output` | 输出文件路径 |

### `crop`

从图片中裁切矩形区域。

```bash
panimg crop photo.jpg --x 100 --y 100 --width 400 --height 300 -o cropped.jpg
```

| 选项 | 说明 |
|------|------|
| `--x` | 左偏移量（像素） |
| `--y` | 上偏移量（像素） |
| `--width` | 裁切宽度（像素） |
| `--height` | 裁切高度（像素） |
| `-o`, `--output` | 输出文件路径 |

### `rotate`

旋转 90、180 或 270 度。

```bash
panimg rotate photo.jpg --angle 90 -o rotated.jpg
```

| 选项 | 说明 |
|------|------|
| `--angle` | 旋转角度：`90`、`180` 或 `270` |
| `-o`, `--output` | 输出文件路径 |

### `flip`

水平或垂直翻转。

```bash
panimg flip photo.jpg --direction horizontal -o flipped.jpg
```

| 选项 | 说明 |
|------|------|
| `--direction` | 翻转方向：`horizontal` 或 `vertical` |
| `-o`, `--output` | 输出文件路径 |

### `auto-orient`

根据 EXIF 方向标签自动旋转，然后移除该标签。

```bash
panimg auto-orient photo.jpg -o oriented.jpg
```

---

## 色彩调整

### `brightness`

调整图片亮度。

```bash
panimg brightness photo.jpg --value 20 -o brighter.jpg
panimg brightness photo.jpg --value -10 -o darker.jpg
```

| 选项 | 说明 |
|------|------|
| `--value` | 亮度调整值（正值 = 更亮，负值 = 更暗） |

### `contrast`

调整图片对比度。

```bash
panimg contrast photo.jpg --value 1.5 -o enhanced.jpg
```

| 选项 | 说明 |
|------|------|
| `--value` | 对比度乘数（1.0 = 不变，>1 = 对比度更强） |

### `hue-rotate`

色相旋转。

```bash
panimg hue-rotate photo.jpg --degrees 90 -o shifted.jpg
```

| 选项 | 说明 |
|------|------|
| `--degrees` | 色相旋转角度 |

### `saturate`

调整色彩饱和度。

```bash
panimg saturate photo.jpg --factor 1.5 -o vivid.jpg
panimg saturate photo.jpg --factor 0.5 -o muted.jpg
```

| 选项 | 说明 |
|------|------|
| `--factor` | 饱和度乘数（1.0 = 不变，0 = 灰度，>1 = 更鲜艳） |

### `grayscale`

转为灰度。

```bash
panimg grayscale photo.jpg -o bw.jpg
```

### `invert`

反转所有色彩。

```bash
panimg invert photo.jpg -o inverted.jpg
```

### `sepia`

应用复古棕褐色调效果。

```bash
panimg sepia photo.jpg -o vintage.jpg
```

### `tint`

以指定色彩着色。

```bash
panimg tint photo.jpg --color red --strength 0.3 -o warm.jpg
panimg tint photo.jpg --color '#FF6600' --strength 0.5 -o orange.jpg
```

| 选项 | 说明 |
|------|------|
| `--color` | 着色颜色（颜色名称或十六进制值） |
| `--strength` | 着色强度，范围 0.0 到 1.0 |

### `posterize`

减少每通道色阶数。

```bash
panimg posterize photo.jpg --levels 4 -o poster.jpg
```

| 选项 | 说明 |
|------|------|
| `--levels` | 每通道的色阶数 |

---

## 滤镜

### `blur`

应用高斯模糊。

```bash
panimg blur photo.jpg --sigma 3.0 -o blurred.jpg
```

| 选项 | 说明 |
|------|------|
| `--sigma` | 模糊半径（值越大模糊越强） |

### `sharpen`

使用非锐化蒙版进行锐化。

```bash
panimg sharpen photo.jpg --sigma 1.5 -o sharp.jpg
```

| 选项 | 说明 |
|------|------|
| `--sigma` | 锐化强度 |

### `edge-detect`

使用 Laplacian 核进行边缘检测。

```bash
panimg edge-detect photo.jpg -o edges.jpg
```

### `emboss`

应用浮雕效果。

```bash
panimg emboss photo.jpg -o embossed.jpg
```

---

## 绘制与合成

### `draw`

在图片上绘制图形。

```bash
panimg draw photo.jpg --shape rect --x 10 --y 10 --width 100 --height 50 --color red -o annotated.jpg
panimg draw photo.jpg --shape circle --cx 200 --cy 200 --radius 50 --color blue --fill -o marked.jpg
panimg draw photo.jpg --shape line --x1 0 --y1 0 --x2 100 --y2 100 --color white -o lined.jpg
```

| 选项 | 说明 |
|------|------|
| `--shape` | 图形类型：`rect`、`circle`、`line` |
| `--x`, `--y` | 位置（矩形） |
| `--cx`, `--cy` | 中心位置（圆形） |
| `--x1`, `--y1`, `--x2`, `--y2` | 起点/终点（线段） |
| `--width`, `--height` | 尺寸（矩形） |
| `--radius` | 半径（圆形） |
| `--color` | 图形颜色（颜色名称或十六进制值） |
| `--fill` | 填充图形（标志） |

### `text`

绘制文字，支持内嵌或自定义字体。

```bash
panimg text photo.jpg --content "Hello World" --size 48 --color white --position center -o titled.jpg
panimg text photo.jpg --content "© 2026" --size 16 --color '#FFFFFF80' --position bottom-right -o stamped.jpg
```

| 选项 | 说明 |
|------|------|
| `--content` | 要渲染的文字内容 |
| `--size` | 字号（像素） |
| `--color` | 文字颜色（颜色名称、十六进制值或带透明度的十六进制值） |
| `--position` | 放置位置：`center`、`top-left`、`top-right`、`bottom-left`、`bottom-right` |
| `--font` | 自定义字体文件路径（可选） |

### `overlay`

将一张图片叠加合成到另一张图片上。

```bash
panimg overlay base.jpg --layer watermark.png --position bottom-right --opacity 0.5 -o watermarked.jpg
```

| 选项 | 说明 |
|------|------|
| `--layer` | 叠加图片的路径 |
| `--position` | 放置位置 |
| `--opacity` | 叠加不透明度，范围 0.0 到 1.0 |

### `trim`

自动裁切空白或相似色边框。

```bash
panimg trim photo.jpg -o trimmed.jpg
```

### `diff`

比较两张图片并可视化差异。

```bash
panimg diff before.png after.png -o diff.png
```

---

## 动画

### `frames`

从动态 GIF 中提取单帧。

```bash
panimg frames animation.gif --output-dir ./frames
```

| 选项 | 说明 |
|------|------|
| `--output-dir` | 保存提取帧的目录 |

### `animate`

将多张图片组合成动态 GIF。

```bash
panimg animate 'frames/*.png' -o animation.gif --delay 100
```

| 选项 | 说明 |
|------|------|
| `--delay` | 帧延迟（毫秒） |
| `-o`, `--output` | 输出 GIF 文件路径 |

### `gif-speed`

调整动画播放速度。

```bash
panimg gif-speed animation.gif -o fast.gif --speed 2.0
panimg gif-speed animation.gif -o slow.gif --speed 0.5
```

| 选项 | 说明 |
|------|------|
| `--speed` | 速度乘数（2.0 = 两倍速，0.5 = 半速） |

---

## 工作流

### `pipeline`

在单次读写中执行多个操作。

```bash
panimg pipeline photo.jpg -o result.jpg --steps "resize --width 800 | blur --sigma 1.5 | grayscale"
panimg pipeline photo.jpg -o result.jpg --steps "brightness --value 10 | contrast --value 1.2 | sharpen --sigma 1.0"
```

| 选项 | 说明 |
|------|------|
| `--steps` | 以管道符分隔的操作列表 |
| `-o`, `--output` | 输出文件路径 |

### `batch`

以 glob 模式匹配多个文件进行批量处理，支持并行执行。

```bash
panimg batch convert 'photos/*.png' --output-dir ./webp --to webp --quality 80
panimg batch resize 'photos/*.jpg' --output-dir ./thumbs --width 200
```

| 选项 | 说明 |
|------|------|
| `--output-dir` | 输出文件目录 |
| `--to` | 目标格式（用于 `batch convert`） |
| 其他选项 | 命令特定选项将被透传 |
