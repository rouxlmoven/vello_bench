// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Tauri commands for benchmark operations.

use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use tokio::sync::Mutex;
use vello_bench_core::{BenchRunner, BenchmarkResult, PlatformInfo, SimdLevel};
use vello_bench_core::data::get_data_items;

/// Mutex to ensure only one benchmark runs at a time.
static BENCHMARK_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

/// Benchmark info for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkInfo {
    pub id: String,
    pub category: String,
    pub name: String,
    pub simd_variant: String,
}

/// SIMD level info for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimdLevelInfo {
    pub id: String,
    pub name: String,
}

/// Get list of available benchmarks.
#[tauri::command]
pub fn list_benchmarks() -> Vec<BenchmarkInfo> {
    let mut benchmarks = vec![];

    // Fine benchmarks (not data-dependent)
    for name in ["opaque_short", "opaque_long", "transparent_short", "transparent_long"] {
        benchmarks.push(BenchmarkInfo {
            id: format!("fine/fill/{}", name),
            category: "fine/fill".into(),
            name: name.into(),
            simd_variant: "u8".into(),
        });
    }

    for name in ["linear_opaque", "radial_opaque"] {
        benchmarks.push(BenchmarkInfo {
            id: format!("fine/gradient/{}", name),
            category: "fine/gradient".into(),
            name: name.into(),
            simd_variant: "u8".into(),
        });
    }

    for name in ["no_transform", "scale"] {
        benchmarks.push(BenchmarkInfo {
            id: format!("fine/image/{}", name),
            category: "fine/image".into(),
            name: name.into(),
            simd_variant: "u8".into(),
        });
    }

    benchmarks.push(BenchmarkInfo {
        id: "fine/pack/block".into(),
        category: "fine/pack".into(),
        name: "block".into(),
        simd_variant: "u8".into(),
    });

    // Data-driven benchmarks (tile, flatten, strip)
    for item in get_data_items() {
        benchmarks.push(BenchmarkInfo {
            id: format!("tile/{}", item.name),
            category: "tile".into(),
            name: item.name.clone(),
            simd_variant: "native".into(),
        });

        benchmarks.push(BenchmarkInfo {
            id: format!("flatten/{}", item.name),
            category: "flatten".into(),
            name: item.name.clone(),
            simd_variant: "native".into(),
        });

        benchmarks.push(BenchmarkInfo {
            id: format!("strip/{}", item.name),
            category: "strip".into(),
            name: item.name.clone(),
            simd_variant: "native".into(),
        });
    }

    // Integration benchmarks
    benchmarks.push(BenchmarkInfo {
        id: "integration/images_overlapping".into(),
        category: "integration".into(),
        name: "images_overlapping".into(),
        simd_variant: "native".into(),
    });

    benchmarks
}

/// Get available SIMD levels.
#[tauri::command]
pub fn get_simd_levels() -> Vec<SimdLevelInfo> {
    SimdLevel::available()
        .into_iter()
        .map(|l| SimdLevelInfo {
            id: l.suffix().to_string(),
            name: l.display_name().to_string(),
        })
        .collect()
}

/// Get platform info.
#[tauri::command]
pub fn get_platform_info() -> PlatformInfo {
    PlatformInfo::detect()
}

/// Run a single benchmark (async, runs in background thread).
#[tauri::command]
pub async fn run_benchmark(
    id: String,
    simd_level: String,
    warmup_ms: u64,
    measurement_ms: u64,
) -> Option<BenchmarkResult> {
    // Acquire lock to ensure only one benchmark runs at a time
    let _guard = BENCHMARK_LOCK.lock().await;

    // Run the benchmark in a blocking thread to not block the async runtime
    let result = tokio::task::spawn_blocking(move || {
        let runner = BenchRunner::new(warmup_ms, measurement_ms);
        let use_scalar = simd_level == "scalar";

        // Parse the benchmark ID and run the appropriate benchmark
        if id.starts_with("fine/fill/") {
            let name = id.strip_prefix("fine/fill/").unwrap();
            Some(run_fine_fill_benchmark(&runner, name, use_scalar))
        } else if id.starts_with("fine/gradient/") {
            let name = id.strip_prefix("fine/gradient/").unwrap();
            Some(run_fine_gradient_benchmark(&runner, name, use_scalar))
        } else if id.starts_with("fine/image/") {
            let name = id.strip_prefix("fine/image/").unwrap();
            Some(run_fine_image_benchmark(&runner, name, use_scalar))
        } else if id.starts_with("fine/pack/") {
            let name = id.strip_prefix("fine/pack/").unwrap();
            Some(run_fine_pack_benchmark(&runner, name, use_scalar))
        } else if id.starts_with("tile/") {
            let name = id.strip_prefix("tile/").unwrap();
            run_tile_benchmark(&runner, name, use_scalar)
        } else if id.starts_with("flatten/") {
            let name = id.strip_prefix("flatten/").unwrap();
            run_flatten_benchmark(&runner, name, use_scalar)
        } else if id.starts_with("strip/") {
            let name = id.strip_prefix("strip/").unwrap();
            run_strip_benchmark(&runner, name, use_scalar)
        } else if id == "integration/images_overlapping" {
            Some(run_integration_benchmark(&runner, "images_overlapping"))
        } else {
            None
        }
    })
    .await
    .ok()
    .flatten();

    result
}

fn create_empty_result(id: &str, category: &str, name: &str, simd_variant: &str) -> BenchmarkResult {
    BenchmarkResult {
        id: id.to_string(),
        category: category.to_string(),
        name: name.to_string(),
        simd_variant: simd_variant.to_string(),
        statistics: vello_bench_core::Statistics {
            mean_ns: 0.0,
            iterations: 0,
        },
        timestamp_ms: 0,
        platform: PlatformInfo::detect(),
    }
}

// ============================================================================
// Fine benchmarks
// ============================================================================

fn run_fine_fill_benchmark(runner: &BenchRunner, name: &str, use_scalar: bool) -> BenchmarkResult {
    use vello_common::color::palette::css::ROYAL_BLUE;
    use vello_common::fearless_simd::Fallback;
    use vello_common::paint::{Paint, PremulColor};
    use vello_common::peniko::{BlendMode, Compose, Mix};
    use vello_cpu::fine::{Fine, U8Kernel};
    use vello_cpu::Level;

    let blend = BlendMode::new(Mix::Normal, Compose::SrcOver);

    let width = match name {
        "opaque_short" | "transparent_short" => 32,
        _ => 256,
    };

    let alpha = if name.contains("transparent") { 0.3 } else { 1.0 };
    let paint = Paint::Solid(PremulColor::from_alpha_color(ROYAL_BLUE.with_alpha(alpha)));

    if use_scalar {
        let mut fine = Fine::<_, U8Kernel>::new(Fallback::new());
        runner.run(
            &format!("fine/fill/{}", name),
            "fine/fill",
            name,
            "scalar",
            || {
                fine.fill(0, width, &paint, blend, &[], None, None);
                std::hint::black_box(&fine);
            },
        )
    } else {
        run_fine_fill_simd(runner, name, width, &paint, blend)
    }
}

#[cfg(target_arch = "aarch64")]
fn run_fine_fill_simd(
    runner: &BenchRunner,
    name: &str,
    width: usize,
    paint: &vello_common::paint::Paint,
    blend: vello_common::peniko::BlendMode,
) -> BenchmarkResult {
    use vello_cpu::fine::{Fine, U8Kernel};
    use vello_cpu::Level;

    let level = Level::new();
    if let Some(neon) = level.as_neon() {
        let mut fine = Fine::<_, U8Kernel>::new(neon);
        runner.run(
            &format!("fine/fill/{}", name),
            "fine/fill",
            name,
            "neon",
            || {
                fine.fill(0, width, paint, blend, &[], None, None);
                std::hint::black_box(&fine);
            },
        )
    } else {
        create_empty_result(&format!("fine/fill/{}", name), "fine/fill", name, "neon")
    }
}

#[cfg(target_arch = "x86_64")]
fn run_fine_fill_simd(
    runner: &BenchRunner,
    name: &str,
    width: usize,
    paint: &vello_common::paint::Paint,
    blend: vello_common::peniko::BlendMode,
) -> BenchmarkResult {
    use vello_cpu::fine::{Fine, U8Kernel};
    use vello_cpu::Level;

    let level = Level::new();
    if let Some(avx2) = level.as_avx2() {
        let mut fine = Fine::<_, U8Kernel>::new(avx2);
        runner.run(
            &format!("fine/fill/{}", name),
            "fine/fill",
            name,
            "avx2",
            || {
                fine.fill(0, width, paint, blend, &[], None, None);
                std::hint::black_box(&fine);
            },
        )
    } else if let Some(sse42) = level.as_sse42() {
        let mut fine = Fine::<_, U8Kernel>::new(sse42);
        runner.run(
            &format!("fine/fill/{}", name),
            "fine/fill",
            name,
            "sse42",
            || {
                fine.fill(0, width, paint, blend, &[], None, None);
                std::hint::black_box(&fine);
            },
        )
    } else {
        create_empty_result(&format!("fine/fill/{}", name), "fine/fill", name, "avx2")
    }
}

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
fn run_fine_fill_simd(
    runner: &BenchRunner,
    name: &str,
    width: usize,
    paint: &vello_common::paint::Paint,
    blend: vello_common::peniko::BlendMode,
) -> BenchmarkResult {
    use vello_common::fearless_simd::Fallback;
    use vello_cpu::fine::{Fine, U8Kernel};

    let mut fine = Fine::<_, U8Kernel>::new(Fallback::new());
    runner.run(
        &format!("fine/fill/{}", name),
        "fine/fill",
        name,
        "scalar",
        || {
            fine.fill(0, width, paint, blend, &[], None, None);
            std::hint::black_box(&fine);
        },
    )
}

fn run_fine_gradient_benchmark(runner: &BenchRunner, name: &str, use_scalar: bool) -> BenchmarkResult {
    use smallvec::smallvec;
    use vello_common::coarse::WideTile;
    use vello_common::color::palette::css::{BLUE, GREEN, RED, YELLOW};
    use vello_common::color::DynamicColor;
    use vello_common::encode::EncodeExt;
    use vello_common::fearless_simd::Fallback;
    use vello_common::kurbo::{Affine, Point};
    use vello_common::peniko::{BlendMode, ColorStop, ColorStops, Compose, Gradient, GradientKind, Mix};
    use vello_common::tile::Tile;
    use vello_cpu::fine::{Fine, U8Kernel};
    use vello_cpu::peniko::{LinearGradientPosition, RadialGradientPosition};

    let blend = BlendMode::new(Mix::Normal, Compose::SrcOver);

    let stops = ColorStops(smallvec![
        ColorStop { offset: 0.0, color: DynamicColor::from_alpha_color(BLUE) },
        ColorStop { offset: 0.33, color: DynamicColor::from_alpha_color(GREEN) },
        ColorStop { offset: 0.66, color: DynamicColor::from_alpha_color(RED) },
        ColorStop { offset: 1.0, color: DynamicColor::from_alpha_color(YELLOW) },
    ]);

    let kind: GradientKind = match name {
        "radial_opaque" => RadialGradientPosition {
            start_center: Point::new(WideTile::WIDTH as f64 / 2.0, (Tile::HEIGHT / 2) as f64),
            start_radius: 25.0,
            end_center: Point::new(WideTile::WIDTH as f64 / 2.0, (Tile::HEIGHT / 2) as f64),
            end_radius: 75.0,
        }.into(),
        _ => LinearGradientPosition {
            start: Point::new(128.0, 128.0),
            end: Point::new(134.0, 134.0),
        }.into(),
    };

    let grad = Gradient {
        kind,
        stops,
        extend: vello_common::peniko::Extend::Pad,
        ..Default::default()
    };

    let mut paints = vec![];
    let paint = grad.encode_into(&mut paints, Affine::IDENTITY);

    if use_scalar {
        let mut fine = Fine::<_, U8Kernel>::new(Fallback::new());
        runner.run(
            &format!("fine/gradient/{}", name),
            "fine/gradient",
            name,
            "scalar",
            || {
                fine.fill(0, WideTile::WIDTH as usize, &paint, blend, &paints, None, None);
                std::hint::black_box(&fine);
            },
        )
    } else {
        run_fine_gradient_simd(runner, name, &paint, &paints, blend)
    }
}

#[cfg(target_arch = "aarch64")]
fn run_fine_gradient_simd(
    runner: &BenchRunner,
    name: &str,
    paint: &vello_common::paint::Paint,
    paints: &[vello_common::encode::EncodedPaint],
    blend: vello_common::peniko::BlendMode,
) -> BenchmarkResult {
    use vello_common::coarse::WideTile;
    use vello_cpu::fine::{Fine, U8Kernel};
    use vello_cpu::Level;

    let level = Level::new();
    if let Some(neon) = level.as_neon() {
        let mut fine = Fine::<_, U8Kernel>::new(neon);
        runner.run(
            &format!("fine/gradient/{}", name),
            "fine/gradient",
            name,
            "neon",
            || {
                fine.fill(0, WideTile::WIDTH as usize, paint, blend, paints, None, None);
                std::hint::black_box(&fine);
            },
        )
    } else {
        create_empty_result(&format!("fine/gradient/{}", name), "fine/gradient", name, "neon")
    }
}

#[cfg(target_arch = "x86_64")]
fn run_fine_gradient_simd(
    runner: &BenchRunner,
    name: &str,
    paint: &vello_common::paint::Paint,
    paints: &[vello_common::encode::EncodedPaint],
    blend: vello_common::peniko::BlendMode,
) -> BenchmarkResult {
    use vello_common::coarse::WideTile;
    use vello_cpu::fine::{Fine, U8Kernel};
    use vello_cpu::Level;

    let level = Level::new();
    if let Some(avx2) = level.as_avx2() {
        let mut fine = Fine::<_, U8Kernel>::new(avx2);
        runner.run(
            &format!("fine/gradient/{}", name),
            "fine/gradient",
            name,
            "avx2",
            || {
                fine.fill(0, WideTile::WIDTH as usize, paint, blend, paints, None, None);
                std::hint::black_box(&fine);
            },
        )
    } else {
        create_empty_result(&format!("fine/gradient/{}", name), "fine/gradient", name, "avx2")
    }
}

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
fn run_fine_gradient_simd(
    runner: &BenchRunner,
    name: &str,
    paint: &vello_common::paint::Paint,
    paints: &[vello_common::encode::EncodedPaint],
    blend: vello_common::peniko::BlendMode,
) -> BenchmarkResult {
    use vello_common::coarse::WideTile;
    use vello_common::fearless_simd::Fallback;
    use vello_cpu::fine::{Fine, U8Kernel};

    let mut fine = Fine::<_, U8Kernel>::new(Fallback::new());
    runner.run(
        &format!("fine/gradient/{}", name),
        "fine/gradient",
        name,
        "scalar",
        || {
            fine.fill(0, WideTile::WIDTH as usize, paint, blend, paints, None, None);
            std::hint::black_box(&fine);
        },
    )
}

fn run_fine_image_benchmark(runner: &BenchRunner, name: &str, use_scalar: bool) -> BenchmarkResult {
    use std::sync::Arc;
    use vello_common::coarse::WideTile;
    use vello_common::encode::EncodeExt;
    use vello_common::fearless_simd::Fallback;
    use vello_common::kurbo::Affine;
    use vello_common::paint::{Image, ImageSource};
    use vello_common::peniko::{BlendMode, Compose, Extend, ImageQuality, ImageSampler, Mix};
    use vello_common::pixmap::Pixmap;
    use vello_cpu::fine::{Fine, U8Kernel};

    let blend = BlendMode::new(Mix::Normal, Compose::SrcOver);

    let data = include_bytes!("../../vello_bench_core/assets/big_colr.png");
    let pixmap = Pixmap::from_png(&data[..]).unwrap();
    let image = Image {
        image: ImageSource::Pixmap(Arc::new(pixmap)),
        sampler: ImageSampler {
            x_extend: Extend::Pad,
            y_extend: Extend::Pad,
            quality: ImageQuality::Low,
            alpha: 1.0,
        },
    };

    let transform = match name {
        "scale" => Affine::scale(3.0),
        _ => Affine::IDENTITY,
    };

    let mut paints = vec![];
    let paint = image.encode_into(&mut paints, transform);

    if use_scalar {
        let mut fine = Fine::<_, U8Kernel>::new(Fallback::new());
        runner.run(
            &format!("fine/image/{}", name),
            "fine/image",
            name,
            "scalar",
            || {
                fine.fill(0, WideTile::WIDTH as usize, &paint, blend, &paints, None, None);
                std::hint::black_box(&fine);
            },
        )
    } else {
        run_fine_image_simd(runner, name, &paint, &paints, blend)
    }
}

#[cfg(target_arch = "aarch64")]
fn run_fine_image_simd(
    runner: &BenchRunner,
    name: &str,
    paint: &vello_common::paint::Paint,
    paints: &[vello_common::encode::EncodedPaint],
    blend: vello_common::peniko::BlendMode,
) -> BenchmarkResult {
    use vello_common::coarse::WideTile;
    use vello_cpu::fine::{Fine, U8Kernel};
    use vello_cpu::Level;

    let level = Level::new();
    if let Some(neon) = level.as_neon() {
        let mut fine = Fine::<_, U8Kernel>::new(neon);
        runner.run(
            &format!("fine/image/{}", name),
            "fine/image",
            name,
            "neon",
            || {
                fine.fill(0, WideTile::WIDTH as usize, paint, blend, paints, None, None);
                std::hint::black_box(&fine);
            },
        )
    } else {
        create_empty_result(&format!("fine/image/{}", name), "fine/image", name, "neon")
    }
}

#[cfg(target_arch = "x86_64")]
fn run_fine_image_simd(
    runner: &BenchRunner,
    name: &str,
    paint: &vello_common::paint::Paint,
    paints: &[vello_common::encode::EncodedPaint],
    blend: vello_common::peniko::BlendMode,
) -> BenchmarkResult {
    use vello_common::coarse::WideTile;
    use vello_cpu::fine::{Fine, U8Kernel};
    use vello_cpu::Level;

    let level = Level::new();
    if let Some(avx2) = level.as_avx2() {
        let mut fine = Fine::<_, U8Kernel>::new(avx2);
        runner.run(
            &format!("fine/image/{}", name),
            "fine/image",
            name,
            "avx2",
            || {
                fine.fill(0, WideTile::WIDTH as usize, paint, blend, paints, None, None);
                std::hint::black_box(&fine);
            },
        )
    } else {
        create_empty_result(&format!("fine/image/{}", name), "fine/image", name, "avx2")
    }
}

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
fn run_fine_image_simd(
    runner: &BenchRunner,
    name: &str,
    paint: &vello_common::paint::Paint,
    paints: &[vello_common::encode::EncodedPaint],
    blend: vello_common::peniko::BlendMode,
) -> BenchmarkResult {
    use vello_common::coarse::WideTile;
    use vello_common::fearless_simd::Fallback;
    use vello_cpu::fine::{Fine, U8Kernel};

    let mut fine = Fine::<_, U8Kernel>::new(Fallback::new());
    runner.run(
        &format!("fine/image/{}", name),
        "fine/image",
        name,
        "scalar",
        || {
            fine.fill(0, WideTile::WIDTH as usize, paint, blend, paints, None, None);
            std::hint::black_box(&fine);
        },
    )
}

fn run_fine_pack_benchmark(runner: &BenchRunner, name: &str, use_scalar: bool) -> BenchmarkResult {
    use vello_common::coarse::WideTile;
    use vello_common::fearless_simd::Fallback;
    use vello_common::tile::Tile;
    use vello_cpu::fine::{Fine, U8Kernel, SCRATCH_BUF_SIZE};
    use vello_cpu::region::Regions;

    if use_scalar {
        let fine = Fine::<_, U8Kernel>::new(Fallback::new());
        runner.run(
            &format!("fine/pack/{}", name),
            "fine/pack",
            name,
            "scalar",
            || {
                let mut buf = vec![0; SCRATCH_BUF_SIZE];
                let mut regions = Regions::new(WideTile::WIDTH, Tile::HEIGHT, &mut buf);
                regions.update_regions(|region| {
                    fine.pack(region);
                });
                std::hint::black_box(&regions);
            },
        )
    } else {
        run_fine_pack_simd(runner, name)
    }
}

#[cfg(target_arch = "aarch64")]
fn run_fine_pack_simd(runner: &BenchRunner, name: &str) -> BenchmarkResult {
    use vello_common::coarse::WideTile;
    use vello_common::tile::Tile;
    use vello_cpu::fine::{Fine, U8Kernel, SCRATCH_BUF_SIZE};
    use vello_cpu::region::Regions;
    use vello_cpu::Level;

    let level = Level::new();
    if let Some(neon) = level.as_neon() {
        let fine = Fine::<_, U8Kernel>::new(neon);
        runner.run(
            &format!("fine/pack/{}", name),
            "fine/pack",
            name,
            "neon",
            || {
                let mut buf = vec![0; SCRATCH_BUF_SIZE];
                let mut regions = Regions::new(WideTile::WIDTH, Tile::HEIGHT, &mut buf);
                regions.update_regions(|region| {
                    fine.pack(region);
                });
                std::hint::black_box(&regions);
            },
        )
    } else {
        create_empty_result(&format!("fine/pack/{}", name), "fine/pack", name, "neon")
    }
}

#[cfg(target_arch = "x86_64")]
fn run_fine_pack_simd(runner: &BenchRunner, name: &str) -> BenchmarkResult {
    use vello_common::coarse::WideTile;
    use vello_common::tile::Tile;
    use vello_cpu::fine::{Fine, U8Kernel, SCRATCH_BUF_SIZE};
    use vello_cpu::region::Regions;
    use vello_cpu::Level;

    let level = Level::new();
    if let Some(avx2) = level.as_avx2() {
        let fine = Fine::<_, U8Kernel>::new(avx2);
        runner.run(
            &format!("fine/pack/{}", name),
            "fine/pack",
            name,
            "avx2",
            || {
                let mut buf = vec![0; SCRATCH_BUF_SIZE];
                let mut regions = Regions::new(WideTile::WIDTH, Tile::HEIGHT, &mut buf);
                regions.update_regions(|region| {
                    fine.pack(region);
                });
                std::hint::black_box(&regions);
            },
        )
    } else {
        create_empty_result(&format!("fine/pack/{}", name), "fine/pack", name, "avx2")
    }
}

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
fn run_fine_pack_simd(runner: &BenchRunner, name: &str) -> BenchmarkResult {
    use vello_common::coarse::WideTile;
    use vello_common::fearless_simd::Fallback;
    use vello_common::tile::Tile;
    use vello_cpu::fine::{Fine, U8Kernel, SCRATCH_BUF_SIZE};
    use vello_cpu::region::Regions;

    let fine = Fine::<_, U8Kernel>::new(Fallback::new());
    runner.run(
        &format!("fine/pack/{}", name),
        "fine/pack",
        name,
        "scalar",
        || {
            let mut buf = vec![0; SCRATCH_BUF_SIZE];
            let mut regions = Regions::new(WideTile::WIDTH, Tile::HEIGHT, &mut buf);
            regions.update_regions(|region| {
                fine.pack(region);
            });
            std::hint::black_box(&regions);
        },
    )
}

// ============================================================================
// Data-driven benchmarks (tile, flatten, strip)
// ============================================================================

fn run_tile_benchmark(runner: &BenchRunner, name: &str, _use_scalar: bool) -> Option<BenchmarkResult> {
    use vello_common::tile::Tiles;
    use vello_cpu::Level;

    let items = get_data_items();
    let item = items.iter().find(|i| i.name == name)?;
    let lines = item.lines();

    // Tile benchmark always uses Level::new() for now
    let result = runner.run(
        &format!("tile/{}", name),
        "tile",
        name,
        "native",
        || {
            let mut tiler = Tiles::new(Level::new());
            tiler.make_tiles_analytic_aa(&lines, item.width, item.height);
            std::hint::black_box(&tiler);
        },
    );

    Some(result)
}

fn run_flatten_benchmark(runner: &BenchRunner, name: &str, _use_scalar: bool) -> Option<BenchmarkResult> {
    use vello_common::flatten::{FlattenCtx, Line};
    use vello_common::kurbo::Affine;
    use vello_cpu::Level;

    let items = get_data_items();
    let item = items.iter().find(|i| i.name == name)?;

    let result = runner.run(
        &format!("flatten/{}", name),
        "flatten",
        name,
        "native",
        || {
            let mut line_buf: Vec<Line> = vec![];
            let mut ctx = FlattenCtx::default();

            for path in &item.fills {
                vello_common::flatten::fill(
                    Level::new(),
                    &path.path,
                    path.transform,
                    &mut line_buf,
                    &mut ctx,
                );
            }

            for path in &item.strokes {
                let stroke = vello_common::kurbo::Stroke {
                    width: path.stroke_width as f64,
                    ..Default::default()
                };
                vello_common::flatten::stroke(
                    Level::new(),
                    &path.path,
                    &stroke,
                    path.transform,
                    &mut line_buf,
                    &mut ctx,
                    &mut vello_common::kurbo::StrokeCtx::default(),
                );
            }

            std::hint::black_box(&line_buf);
        },
    );

    Some(result)
}

fn run_strip_benchmark(runner: &BenchRunner, name: &str, _use_scalar: bool) -> Option<BenchmarkResult> {
    use vello_common::peniko::Fill;
    use vello_common::strip::Strip;
    use vello_cpu::Level;

    let items = get_data_items();
    let item = items.iter().find(|i| i.name == name)?;
    let lines = item.lines();
    let tiles = item.sorted_tiles();

    let result = runner.run(
        &format!("strip/{}", name),
        "strip",
        name,
        "native",
        || {
            let mut strip_buf: Vec<Strip> = vec![];
            let mut alpha_buf: Vec<u8> = vec![];

            vello_common::strip::render(
                Level::fallback(),
                &tiles,
                &mut strip_buf,
                &mut alpha_buf,
                Fill::NonZero,
                None,
                &lines,
            );

            std::hint::black_box(&strip_buf);
        },
    );

    Some(result)
}

// ============================================================================
// Integration benchmarks
// ============================================================================

fn run_integration_benchmark(runner: &BenchRunner, name: &str) -> BenchmarkResult {
    use std::sync::Arc;
    use vello_common::kurbo::{Affine, Rect};
    use vello_common::paint::{Image, ImageSource};
    use vello_common::peniko::{Extend, ImageQuality, ImageSampler};
    use vello_common::pixmap::Pixmap;
    use vello_cpu::color::AlphaColor;
    use vello_cpu::RenderContext;

    let image_data = include_bytes!("../../vello_bench_core/assets/splash-flower.jpg");
    let image = image::load_from_memory(image_data).expect("Failed to decode image");
    let width = image.width();
    let height = image.height();
    let rgba_data = image.into_rgba8().into_vec();

    let mut may_have_opacities = false;
    #[allow(clippy::cast_possible_truncation)]
    let pixmap = Pixmap::from_parts_with_opacity(
        rgba_data
            .chunks_exact(4)
            .map(|rgba| {
                let alpha = rgba[3];
                if alpha != 255 {
                    may_have_opacities = true;
                }
                AlphaColor::from_rgba8(rgba[0], rgba[1], rgba[2], alpha)
                    .premultiply()
                    .to_rgba8()
            })
            .collect(),
        width as u16,
        height as u16,
        may_have_opacities,
    );

    let flower_image = ImageSource::Pixmap(Arc::new(pixmap));

    const VIEWPORT_WIDTH: u16 = 1280;
    const VIEWPORT_HEIGHT: u16 = 960;

    let ImageSource::Pixmap(ref image_pixmap) = flower_image else {
        panic!("Expected Pixmap");
    };
    let original_width = f64::from(image_pixmap.width());
    let original_height = f64::from(image_pixmap.height());
    let image_count = VIEWPORT_WIDTH / 256;

    let mut renderer = RenderContext::new(VIEWPORT_WIDTH, VIEWPORT_HEIGHT);
    let mut out_pixmap = Pixmap::new(VIEWPORT_WIDTH, VIEWPORT_HEIGHT);

    runner.run(
        &format!("integration/{}", name),
        "integration",
        name,
        "native",
        || {
            renderer.reset();

            for i in (1..=image_count).rev() {
                let w = 256.0 * i as f64;
                let scale = w / original_width;
                let h = original_height * scale;

                renderer.set_paint_transform(Affine::scale(scale));
                renderer.set_paint(Image {
                    image: flower_image.clone(),
                    sampler: ImageSampler {
                        x_extend: Extend::Pad,
                        y_extend: Extend::Pad,
                        quality: ImageQuality::Medium,
                        alpha: 1.0,
                    },
                });
                renderer.fill_rect(&Rect::new(0.0, 0.0, w, h));
            }

            renderer.flush();
            renderer.render_to_pixmap(&mut out_pixmap);
            std::hint::black_box(&out_pixmap);
        },
    )
}
