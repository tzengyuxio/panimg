# AI 代理集成

panimg 专为与 AI 代理、LLM 工具调用管线及自动化脚本无缝集成而设计。每个命令都支持结构化 JSON 输出、参数自省和无副作用预览。

## 功能发现

以编程方式查询所有支持的命令、格式和功能：

```bash
panimg --capabilities --format json
```

该命令返回一个 JSON 对象，描述每个可用命令、支持的输入/输出格式以及已启用的 feature flag。AI 代理可利用此信息在运行时确定可执行的操作。

## Schema 自省

以 JSON 格式获取任意命令的参数定义：

```bash
panimg resize --schema
panimg convert --schema
panimg pipeline --schema
```

Schema 输出包含参数名称、类型、默认值和有效值——适用于在 LLM 框架中生成工具调用的函数定义。

## Dry-Run 模式

预览任意操作而不写入文件：

```bash
panimg resize photo.jpg --width 800 -o out.jpg --dry-run --format json
```

返回一个 JSON 对象，描述该操作*将会*执行的内容（输出尺寸、格式、估算文件大小），而不产生任何副作用。这使代理能够在执行前规划和验证操作。

## 结构化 JSON 输出

所有命令都支持 `--format json` 以获取机器可读的输出：

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

## 退出码

panimg 使用特定的退出码，以便代理可以通过程序判断执行结果：

| 退出码 | 含义 |
|--------|------|
| 0 | 成功 |
| 1 | 一般错误 |
| 2 | 输入文件错误（找不到文件、权限不足、解码失败） |
| 3 | 输出问题（写入失败、文件已存在） |
| 4 | 不支持的格式 |
| 5 | 参数错误 |

## 错误输出

错误信息具有结构化格式，并包含可操作的建议：

```bash
$ panimg convert missing.png out.webp
error: file not found: missing.png
  hint: check that the file path is correct
```

使用 `--format json` 时：

```json
{
  "error": "file_not_found",
  "path": "missing.png",
  "suggestion": "check that the file path is correct"
}
```

## 示例：LLM 工具调用集成

典型的代理工作流：

1. **发现**可用操作：`panimg --capabilities --format json`
2. **检查**所选命令的参数：`panimg resize --schema`
3. **预览**操作：`panimg resize photo.jpg --width 800 -o out.jpg --dry-run --format json`
4. **执行**操作：`panimg resize photo.jpg --width 800 -o out.jpg --format json`
5. **验证**结果：`panimg info out.jpg --format json`
