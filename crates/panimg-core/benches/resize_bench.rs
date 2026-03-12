use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use image::{DynamicImage, RgbaImage};
use panimg_core::ops::resize::{FitMode, ResizeFilter, ResizeOp};
use panimg_core::ops::Operation;

/// Generate a synthetic test image of the given dimensions.
fn generate_test_image(width: u32, height: u32) -> DynamicImage {
    let mut img = RgbaImage::new(width, height);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        *pixel = image::Rgba([
            (x % 256) as u8,
            (y % 256) as u8,
            ((x + y) % 256) as u8,
            255,
        ]);
    }
    DynamicImage::ImageRgba8(img)
}

fn bench_resize(c: &mut Criterion) {
    let cases: Vec<(&str, u32, u32, u32)> = vec![
        ("768→384", 768, 512, 384),
        ("1920→800", 1920, 1080, 800),
        ("3840→800", 3840, 2160, 800),
        ("3840→1920", 3840, 2160, 1920),
    ];

    let mut group = c.benchmark_group("resize_lanczos3");

    for (label, src_w, src_h, dst_w) in &cases {
        let img = generate_test_image(*src_w, *src_h);
        let pixels = (*src_w as u64) * (*src_h as u64);
        group.throughput(Throughput::Elements(pixels));

        group.bench_with_input(BenchmarkId::new("contain", label), &img, |b, img| {
            let op =
                ResizeOp::new(Some(*dst_w), None, FitMode::Contain, ResizeFilter::Lanczos3)
                    .unwrap();
            b.iter(|| {
                op.apply(img.clone()).unwrap();
            });
        });
    }

    group.finish();

    // Compare filters at a fixed size
    let mut filter_group = c.benchmark_group("resize_filters_1920→800");
    let img_1080p = generate_test_image(1920, 1080);
    let pixels = 1920u64 * 1080;
    filter_group.throughput(Throughput::Elements(pixels));

    let filters = [
        ("lanczos3", ResizeFilter::Lanczos3),
        ("catmull-rom", ResizeFilter::CatmullRom),
        ("linear", ResizeFilter::Linear),
        ("nearest", ResizeFilter::Nearest),
    ];

    for (name, filter) in &filters {
        filter_group.bench_with_input(BenchmarkId::new("filter", name), &img_1080p, |b, img| {
            let op = ResizeOp::new(Some(800), None, FitMode::Contain, *filter).unwrap();
            b.iter(|| {
                op.apply(img.clone()).unwrap();
            });
        });
    }

    filter_group.finish();
}

criterion_group!(benches, bench_resize);
criterion_main!(benches);
