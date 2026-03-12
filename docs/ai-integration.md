# AI Agent Integration

panimg is designed for seamless integration with AI agents, LLM tool-use pipelines, and automation scripts. Every command supports structured JSON output, parameter introspection, and side-effect-free previews.

## Capabilities Discovery

Query all supported commands, formats, and features programmatically:

```bash
panimg --capabilities --format json
```

This returns a JSON object describing every available command, supported input/output formats, and enabled feature flags. AI agents can use this to determine what operations are available at runtime.

## Schema Introspection

Get the parameter definitions for any command as JSON:

```bash
panimg resize --schema
panimg convert --schema
panimg pipeline --schema
```

The schema output includes parameter names, types, defaults, and valid values — suitable for generating tool-use function definitions in LLM frameworks.

## Dry-Run Mode

Preview any operation without writing files:

```bash
panimg resize photo.jpg --width 800 -o out.jpg --dry-run --format json
```

Returns a JSON object describing what the operation *would* do (output dimensions, format, estimated file size) without any side effects. This allows agents to plan and validate operations before committing.

## Structured JSON Output

All commands support `--format json` for machine-readable output:

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

## Exit Codes

panimg uses specific exit codes so agents can programmatically determine the outcome:

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Input file error (not found, permission denied, decode failure) |
| 3 | Output issue (write failure, file exists) |
| 4 | Unsupported format |
| 5 | Bad arguments |

## Error Output

Errors are structured and include actionable suggestions:

```bash
$ panimg convert missing.png out.webp
error: file not found: missing.png
  hint: check that the file path is correct
```

With `--format json`:

```json
{
  "error": "file_not_found",
  "path": "missing.png",
  "suggestion": "check that the file path is correct"
}
```

## Example: LLM Tool-Use Integration

A typical agent workflow:

1. **Discover** available operations: `panimg --capabilities --format json`
2. **Inspect** the chosen command's parameters: `panimg resize --schema`
3. **Preview** the operation: `panimg resize photo.jpg --width 800 -o out.jpg --dry-run --format json`
4. **Execute** the operation: `panimg resize photo.jpg --width 800 -o out.jpg --format json`
5. **Verify** the result: `panimg info out.jpg --format json`
