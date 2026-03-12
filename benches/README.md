# panimg Benchmarks

Performance benchmarks comparing panimg against ImageMagick, libvips, and GraphicsMagick.

## Prerequisites

```bash
brew install hyperfine vips graphicsmagick imagemagick
cargo build --release
```

## Quick Start

```bash
# 1. Download test images and generate synthetic images
bash benches/scripts/setup.sh

# 2. Run all benchmarks
bash benches/scripts/bench_single.sh    # 10 single-operation comparisons
bash benches/scripts/bench_batch.sh     # Batch processing (24 Kodak images)
bash benches/scripts/bench_pipeline.sh  # Pipeline vs chained commands
bash benches/scripts/bench_memory.sh    # Peak memory usage
bash benches/scripts/bench_quality.sh   # Output quality comparison

# 3. Generate report
python3 benches/scripts/report.py
# → benches/results/REPORT.md
```

## Criterion Micro-benchmarks

```bash
cargo bench --bench resize_bench
```

Results are saved to `target/criterion/`.

## Directory Structure

```
benches/
├── images/          # Test images (git-ignored)
├── output/          # Benchmark output files (git-ignored)
├── results/         # JSON results + REPORT.md
├── scripts/         # Benchmark and setup scripts
│   ├── setup.sh
│   ├── bench_single.sh
│   ├── bench_batch.sh
│   ├── bench_pipeline.sh
│   ├── bench_memory.sh
│   ├── bench_quality.sh
│   └── report.py
└── criterion/       # (reserved for criterion config)
```
