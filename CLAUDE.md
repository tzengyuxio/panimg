# panimg

Rust CLI tool for batch image processing (convert, resize, compress, text overlay, set-density).

## Workspace

```
crates/
  pan-common/    # shared types and utilities (published to crates.io)
  panimg-core/   # image processing library
  panimg-cli/    # CLI binary
```

## Build & Test

```bash
cargo build                    # build all crates
cargo test                     # run all tests
cargo clippy --all-targets     # lint
cargo fmt --check              # check formatting
```

## Conventions

- Dual-licensed: MIT OR Apache-2.0
- Error handling: `thiserror` + `miette`
- Optional codecs gated behind cargo features (avif, jxl, svg, heif)
