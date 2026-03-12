# AI 代理整合

panimg 專為與 AI 代理、LLM 工具使用管線及自動化腳本無縫整合而設計。每個命令都支援結構化 JSON 輸出、參數自省與無副作用預覽。

## 功能探索

以程式化方式查詢所有支援的命令、格式與功能：

```bash
panimg --capabilities --format json
```

這會回傳一個 JSON 物件，描述所有可用命令、支援的輸入/輸出格式以及已啟用的 feature flag。AI 代理可利用此功能在執行時期判斷哪些操作可用。

## Schema 自省

以 JSON 格式取得任意命令的參數定義：

```bash
panimg resize --schema
panimg convert --schema
panimg pipeline --schema
```

Schema 輸出包含參數名稱、型別、預設值與合法值——適合在 LLM 框架中產生工具使用的函式定義。

## Dry-Run 模式

預覽任何操作而不寫入檔案：

```bash
panimg resize photo.jpg --width 800 -o out.jpg --dry-run --format json
```

回傳一個 JSON 物件，描述該操作*將會*執行的內容（輸出尺寸、格式、預估檔案大小），不產生任何副作用。這讓代理可以在實際執行前規劃與驗證操作。

## 結構化 JSON 輸出

所有命令都支援 `--format json` 以取得機器可讀的輸出：

```bash
panimg info photo.jpg --format json --fields width,height,format
```

```json
{
  "width": 4032,
  "height": 3024,
  "format": "jpeg"
}
```

## Exit Code

panimg 使用特定的 exit code，讓代理可以程式化地判斷執行結果：

| 代碼 | 意義 |
|------|------|
| 0 | 成功 |
| 1 | 一般錯誤 |
| 2 | 輸入檔案錯誤（找不到檔案、權限不足、解碼失敗） |
| 3 | 輸出問題（寫入失敗、檔案已存在） |
| 4 | 不支援的格式 |
| 5 | 參數錯誤 |

## 錯誤輸出

錯誤訊息為結構化格式，並包含可操作的建議：

```bash
$ panimg convert missing.png out.webp
error: file not found: missing.png
  hint: check that the file path is correct
```

使用 `--format json` 時：

```json
{
  "error": "file_not_found",
  "path": "missing.png",
  "suggestion": "check that the file path is correct"
}
```

## 範例：LLM 工具使用整合

典型的代理工作流程：

1. **探索**可用操作：`panimg --capabilities --format json`
2. **檢視**所選命令的參數：`panimg resize --schema`
3. **預覽**操作：`panimg resize photo.jpg --width 800 -o out.jpg --dry-run --format json`
4. **執行**操作：`panimg resize photo.jpg --width 800 -o out.jpg --format json`
5. **驗證**結果：`panimg info out.jpg --format json`
