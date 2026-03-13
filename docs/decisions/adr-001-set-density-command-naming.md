---
id: "ADR-001"
date: 2026-03-14
status: accepted
scope: "panimg-cli"
tags: [cli, naming, resolution]
---

# ADR-001: Name the resolution conversion command `set-density`

## Context

panimg needs a new command to convert image resolution (DPI/DPCM) — both modifying metadata and optionally resampling pixels. The command must support multiple resolution units (dots per inch, dots per centimetre), so the name should be unit-neutral.

## Decision

Use **`set-density`** as the command name.

```bash
panimg set-density input.jpg -o output.jpg --density 100 --unit dpcm
panimg set-density input.jpg -o output.jpg --density 100 --unit dpcm --resample
```

## Consequences

- The name is unit-neutral — "density" is the overarching concept for dpi/dpcm without biasing toward either unit system.
- Aligns with ImageMagick's established `-density` option, reducing cognitive load for users familiar with other image tools.
- The `--resample` flag clearly distinguishes metadata-only vs pixel-resampling modes under the same command.

## Alternatives Considered

| Name | Rejected Because |
|------|-----------------|
| `set-dpi` | Biased toward inches; misleading when the user targets dpcm |
| `dpi` | Same unit bias; too terse to convey the "set" action |
| `resample` | Implies pixel changes; misleading for metadata-only mode |
| `resolution` | Overly generic; could be confused with pixel dimensions (width × height) |
| `set-resolution` | Same ambiguity as `resolution` |
