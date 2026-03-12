# AI エージェント連携

panimg は AI エージェント、LLM ツール使用パイプライン、自動化スクリプトとのシームレスな連携を前提に設計されています。すべてのコマンドが構造化 JSON 出力、パラメータイントロスペクション、副作用なしのプレビューに対応しています。

## 機能の探索

サポートされているすべてのコマンド、フォーマット、機能をプログラム的に照会できます：

```bash
panimg --capabilities --format json
```

利用可能なすべてのコマンド、対応する入出力フォーマット、有効なフィーチャーフラグを記述した JSON オブジェクトが返されます。AI エージェントはこれを使用して、実行時にどの操作が利用可能かを判断できます。

## スキーマイントロスペクション

任意のコマンドのパラメータ定義を JSON として取得できます：

```bash
panimg resize --schema
panimg convert --schema
panimg pipeline --schema
```

スキーマ出力にはパラメータ名、型、デフォルト値、有効な値が含まれます。LLM フレームワークでツール使用の関数定義を生成するのに適しています。

## ドライランモード

ファイルを書き込まずに任意の操作をプレビューできます：

```bash
panimg resize photo.jpg --width 800 -o out.jpg --dry-run --format json
```

操作が*実行された場合に何が起こるか*（出力の寸法、フォーマット、推定ファイルサイズ）を記述した JSON オブジェクトが返されます。副作用はありません。これにより、エージェントは実行前に操作の計画と検証を行えます。

## 構造化 JSON 出力

すべてのコマンドが `--format json` による機械可読な出力に対応しています：

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

## 終了コード

panimg はエージェントがプログラム的に結果を判定できるよう、特定の終了コードを使用します：

| コード | 意味 |
|--------|------|
| 0 | 成功 |
| 1 | 一般エラー |
| 2 | 入力ファイルエラー（ファイルが見つからない、権限拒否、デコード失敗） |
| 3 | 出力の問題（書き込み失敗、ファイルが既に存在） |
| 4 | 非対応フォーマット |
| 5 | 引数エラー |

## エラー出力

エラーは構造化されており、対処可能な提案が含まれます：

```bash
$ panimg convert missing.png out.webp
error: file not found: missing.png
  hint: check that the file path is correct
```

`--format json` を指定した場合：

```json
{
  "error": "file_not_found",
  "path": "missing.png",
  "suggestion": "check that the file path is correct"
}
```

## 使用例：LLM ツール使用連携

一般的なエージェントワークフロー：

1. **探索** — 利用可能な操作を確認：`panimg --capabilities --format json`
2. **検査** — 選択したコマンドのパラメータを確認：`panimg resize --schema`
3. **プレビュー** — 操作をプレビュー：`panimg resize photo.jpg --width 800 -o out.jpg --dry-run --format json`
4. **実行** — 操作を実行：`panimg resize photo.jpg --width 800 -o out.jpg --format json`
5. **検証** — 結果を確認：`panimg info out.jpg --format json`
