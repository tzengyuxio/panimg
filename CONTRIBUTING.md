# Contributing to panimg

Thank you for your interest in contributing to panimg!

## Development Setup

1. Install Rust (stable): https://rustup.rs/
2. Clone the repository:
   ```bash
   git clone https://github.com/tzengyuxio/panimg.git
   cd panimg
   ```
3. Build:
   ```bash
   cargo build
   ```
4. Run tests:
   ```bash
   cargo test --all-features
   ```

## Project Structure

```
panimg/
├── crates/
│   ├── panimg-core/    # Library crate (engine)
│   └── panimg-cli/     # Binary crate (CLI)
```

- **panimg-core**: All image processing logic, format detection, codec registry, operations, and pipeline engine.
- **panimg-cli**: CLI interface using clap, output formatting, and command implementations.

## Code Quality

Before submitting a PR, ensure:

```bash
cargo fmt --check
cargo clippy --all-features -- -D warnings
cargo test --all-features
```

## Guidelines

- Keep the `Operation` trait simple — one operation per struct
- All errors must be structured (`PanimgError`) with suggestions
- New commands must support `--format json`, `--dry-run`, and `--schema`
- Test fixtures should be generated programmatically (no large binary files)
- Feature-gate optional codec dependencies

## Pull Requests

1. Create a feature branch from `main`
2. Write tests for new functionality
3. Ensure CI passes
4. Describe your changes clearly in the PR description

## License

By contributing, you agree that your contributions will be licensed under the same dual license as the project (MIT OR Apache-2.0).
