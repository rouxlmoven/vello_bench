// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Centralized benchmark dispatch - single source of truth for benchmark definitions.

use crate::data::get_data_items;
use crate::result::BenchmarkResult;
use crate::runner::BenchRunner;
use serde::{Deserialize, Serialize};

/// Benchmark info for the frontend/CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkInfo {
    pub id: String,
    pub category: String,
    pub name: String,
}

/// Get the complete list of all available benchmarks.
pub fn get_benchmark_list() -> Vec<BenchmarkInfo> {
    let mut benchmarks = vec![];

    // Fine/Fill benchmarks
    for name in ["opaque_short", "opaque_long", "transparent_short", "transparent_long"] {
        benchmarks.push(BenchmarkInfo {
            id: format!("fine/fill/{}", name),
            category: "fine/fill".into(),
            name: name.into(),
        });
    }

    // Fine/Gradient benchmarks
    for name in ["linear_opaque", "radial_opaque", "sweep_opaque", "many_stops", "transparent"] {
        benchmarks.push(BenchmarkInfo {
            id: format!("fine/gradient/{}", name),
            category: "fine/gradient".into(),
            name: name.into(),
        });
    }

    // Fine/Image benchmarks
    for name in ["no_transform", "scale", "rotate", "quality_medium", "quality_high", "extend_repeat"] {
        benchmarks.push(BenchmarkInfo {
            id: format!("fine/image/{}", name),
            category: "fine/image".into(),
            name: name.into(),
        });
    }

    // Fine/Pack benchmarks
    for name in ["block", "regular"] {
        benchmarks.push(BenchmarkInfo {
            id: format!("fine/pack/{}", name),
            category: "fine/pack".into(),
            name: name.into(),
        });
    }

    // Fine/Strip benchmarks
    for name in ["solid_short", "solid_long"] {
        benchmarks.push(BenchmarkInfo {
            id: format!("fine/strip/{}", name),
            category: "fine/strip".into(),
            name: name.into(),
        });
    }

    // Data-driven benchmarks (tile, flatten, strokes, render_strips)
    for item in get_data_items() {
        benchmarks.push(BenchmarkInfo {
            id: format!("tile/{}", item.name),
            category: "tile".into(),
            name: item.name.clone(),
        });

        benchmarks.push(BenchmarkInfo {
            id: format!("flatten/{}", item.name),
            category: "flatten".into(),
            name: item.name.clone(),
        });

        benchmarks.push(BenchmarkInfo {
            id: format!("strokes/{}", item.name),
            category: "strokes".into(),
            name: item.name.clone(),
        });

        benchmarks.push(BenchmarkInfo {
            id: format!("render_strips/{}", item.name),
            category: "render_strips".into(),
            name: item.name.clone(),
        });
    }

    benchmarks
}

/// Run a benchmark by ID with a specific SIMD level.
/// Returns None if the benchmark ID is not found.
pub fn run_benchmark_by_id(
    runner: &BenchRunner,
    id: &str,
    simd_level: crate::SimdLevel,
) -> Option<BenchmarkResult> {
    // Fine benchmarks
    if let Some(name) = id.strip_prefix("fine/fill/") {
        return Some(run_fine_fill(runner, name, simd_level));
    }
    if let Some(name) = id.strip_prefix("fine/gradient/") {
        return Some(run_fine_gradient(runner, name, simd_level));
    }
    if let Some(name) = id.strip_prefix("fine/image/") {
        return Some(run_fine_image(runner, name, simd_level));
    }
    if let Some(name) = id.strip_prefix("fine/pack/") {
        return Some(run_fine_pack(runner, name, simd_level));
    }
    if let Some(name) = id.strip_prefix("fine/strip/") {
        return Some(run_fine_strip(runner, name, simd_level));
    }
    // Data-driven benchmarks
    if let Some(name) = id.strip_prefix("tile/") {
        return run_tile(runner, name, simd_level);
    }
    if let Some(name) = id.strip_prefix("flatten/") {
        return run_flatten(runner, name, simd_level);
    }
    if let Some(name) = id.strip_prefix("strokes/") {
        return run_strokes(runner, name, simd_level);
    }
    if let Some(name) = id.strip_prefix("render_strips/") {
        return run_render_strips(runner, name, simd_level);
    }

    None
}

// ============================================================================
// Fine/Fill benchmark
// ============================================================================

fn run_fine_fill(runner: &BenchRunner, name: &str, simd_level: crate::SimdLevel) -> BenchmarkResult {
    use vello_common::color::palette::css::ROYAL_BLUE;
    use fearless_simd::dispatch;
    use vello_common::paint::{Paint, PremulColor};
    use vello_common::peniko::{BlendMode, Compose, Mix};
    use vello_cpu::fine::{Fine, U8Kernel};

    let blend = BlendMode::new(Mix::Normal, Compose::SrcOver);

    let width = match name {
        "opaque_short" | "transparent_short" => 32,
        _ => 256,
    };

    let alpha = if name.contains("transparent") { 0.3 } else { 1.0 };
    let paint = Paint::Solid(PremulColor::from_alpha_color(ROYAL_BLUE.with_alpha(alpha)));

    let level = simd_level.to_level().unwrap_or_else(|| vello_cpu::Level::fallback());
    let simd_variant = simd_level.suffix();

    dispatch!(level, simd => {
        let mut fine = Fine::<_, U8Kernel>::new(simd);

        runner.run(
            &format!("fine/fill/{}", name),
            "fine/fill",
            name,
            simd_variant,
            || {
                fine.fill(0, width, &paint, blend, &[], None, None);
                std::hint::black_box(&fine);
            },
        )
    })
}

// ============================================================================
// Fine/Gradient benchmark
// ============================================================================

fn run_fine_gradient(runner: &BenchRunner, name: &str, simd_level: crate::SimdLevel) -> BenchmarkResult {
    use rand::prelude::StdRng;
    use rand::{Rng, SeedableRng};
    use smallvec::{SmallVec, smallvec};
    use vello_common::coarse::WideTile;
    use vello_common::color::palette::css::{BLUE, GREEN, RED, YELLOW};
    use vello_common::color::{AlphaColor, DynamicColor, Srgb};
    use fearless_simd::dispatch;
    use vello_common::encode::EncodeExt;
    use vello_common::kurbo::{Affine, Point};
    use vello_common::peniko::{BlendMode, ColorStop, ColorStops, Compose, Gradient, GradientKind, Mix};
    use vello_common::tile::Tile;
    use vello_cpu::fine::{Fine, U8Kernel};
    use vello_cpu::peniko::{LinearGradientPosition, RadialGradientPosition, SweepGradientPosition};

    const SEED: [u8; 32] = [0; 32];
    let blend = BlendMode::new(Mix::Normal, Compose::SrcOver);

    let stops: ColorStops = match name {
        "many_stops" => {
            let mut vec = SmallVec::new();
            let mut rng = StdRng::from_seed(SEED);
            let max = 120;
            for i in 0..=120 {
                let offset = i as f32 / max as f32;
                let color = DynamicColor::from_alpha_color(AlphaColor::<Srgb>::new([
                    rng.random::<f32>(),
                    rng.random::<f32>(),
                    rng.random::<f32>(),
                    rng.random::<f32>(),
                ]));
                vec.push(ColorStop { offset, color });
            }
            ColorStops(vec)
        }
        "transparent" => ColorStops(smallvec![
            ColorStop { offset: 0.0, color: DynamicColor::from_alpha_color(BLUE) },
            ColorStop { offset: 0.33, color: DynamicColor::from_alpha_color(GREEN.with_alpha(0.5)) },
            ColorStop { offset: 0.66, color: DynamicColor::from_alpha_color(RED) },
            ColorStop { offset: 1.0, color: DynamicColor::from_alpha_color(YELLOW.with_alpha(0.7)) },
        ]),
        _ => ColorStops(smallvec![
            ColorStop { offset: 0.0, color: DynamicColor::from_alpha_color(BLUE) },
            ColorStop { offset: 0.33, color: DynamicColor::from_alpha_color(GREEN) },
            ColorStop { offset: 0.66, color: DynamicColor::from_alpha_color(RED) },
            ColorStop { offset: 1.0, color: DynamicColor::from_alpha_color(YELLOW) },
        ]),
    };

    let kind: GradientKind = match name {
        "radial_opaque" => RadialGradientPosition {
            start_center: Point::new(WideTile::WIDTH as f64 / 2.0, (Tile::HEIGHT / 2) as f64),
            start_radius: 25.0,
            end_center: Point::new(WideTile::WIDTH as f64 / 2.0, (Tile::HEIGHT / 2) as f64),
            end_radius: 75.0,
        }.into(),
        "sweep_opaque" => SweepGradientPosition {
            center: Point::new(WideTile::WIDTH as f64 / 2.0, (Tile::HEIGHT / 2) as f64),
            start_angle: 70.0_f32.to_radians(),
            end_angle: 250.0_f32.to_radians(),
        }.into(),
        _ => LinearGradientPosition {
            start: Point::new(128.0, 128.0),
            end: Point::new(134.0, 134.0),
        }.into(),
    };

    let extend = match name {
        "many_stops" => vello_common::peniko::Extend::Repeat,
        _ => vello_common::peniko::Extend::Pad,
    };

    let grad = Gradient { kind, stops, extend, ..Default::default() };
    let mut paints = vec![];
    let paint = grad.encode_into(&mut paints, Affine::IDENTITY);

    let level = simd_level.to_level().unwrap_or_else(|| vello_cpu::Level::fallback());
    let simd_variant = simd_level.suffix();

    dispatch!(level, simd => {
        let mut fine = Fine::<_, U8Kernel>::new(simd);

        runner.run(
            &format!("fine/gradient/{}", name),
            "fine/gradient",
            name,
            simd_variant,
            || {
                fine.fill(0, WideTile::WIDTH as usize, &paint, blend, &paints, None, None);
                std::hint::black_box(&fine);
            },
        )
    })
}

// ============================================================================
// Fine/Image benchmark
// ============================================================================

fn run_fine_image(runner: &BenchRunner, name: &str, simd_level: crate::SimdLevel) -> BenchmarkResult {
    use std::sync::Arc;
    use vello_common::coarse::WideTile;
    use fearless_simd::dispatch;
    use vello_common::encode::EncodeExt;
    use vello_common::kurbo::{Affine, Point};
    use vello_common::paint::{Image, ImageSource};
    use vello_common::peniko::{BlendMode, Compose, Extend, ImageQuality, ImageSampler, Mix};
    use vello_common::pixmap::Pixmap;
    use vello_common::tile::Tile;
    use vello_cpu::fine::{Fine, U8Kernel};

    let blend = BlendMode::new(Mix::Normal, Compose::SrcOver);

    let quality = match name {
        "quality_medium" => ImageQuality::Medium,
        "quality_high" => ImageQuality::High,
        _ => ImageQuality::Low,
    };

    let extend = match name {
        "extend_repeat" => Extend::Repeat,
        _ => Extend::Pad,
    };

    // Use different images and transforms for different tests
    static COLR_DATA: &[u8] = include_bytes!("../assets/big_colr.png");
    static SMALL_DATA: &[u8] = include_bytes!("../assets/rgb_image_2x2.png");

    let (data, transform): (&[u8], Affine) = match name {
        "extend_repeat" => (SMALL_DATA, Affine::translate((WideTile::WIDTH as f64 / 2.0, 0.0))),
        "scale" | "quality_medium" | "quality_high" => (COLR_DATA, Affine::scale(3.0)),
        "rotate" => (COLR_DATA, Affine::rotate_about(1.0, Point::new(WideTile::WIDTH as f64 / 2.0, Tile::HEIGHT as f64 / 2.0))),
        _ => (COLR_DATA, Affine::IDENTITY),
    };

    let pixmap = Pixmap::from_png(data).unwrap();
    let image = Image {
        image: ImageSource::Pixmap(Arc::new(pixmap)),
        sampler: ImageSampler { x_extend: extend, y_extend: extend, quality, alpha: 1.0 },
    };

    let mut paints = vec![];
    let paint = image.encode_into(&mut paints, transform);

    let level = simd_level.to_level().unwrap_or_else(|| vello_cpu::Level::fallback());
    let simd_variant = simd_level.suffix();

    dispatch!(level, simd => {
        let mut fine = Fine::<_, U8Kernel>::new(simd);

        runner.run(
            &format!("fine/image/{}", name),
            "fine/image",
            name,
            simd_variant,
            || {
                fine.fill(0, WideTile::WIDTH as usize, &paint, blend, &paints, None, None);
                std::hint::black_box(&fine);
            },
        )
    })
}

// ============================================================================
// Fine/Pack benchmark
// ============================================================================

fn run_fine_pack(runner: &BenchRunner, name: &str, simd_level: crate::SimdLevel) -> BenchmarkResult {
    use vello_common::coarse::WideTile;
    use fearless_simd::dispatch;
    use vello_common::tile::Tile;
    use vello_cpu::fine::{Fine, U8Kernel, SCRATCH_BUF_SIZE};
    use vello_cpu::region::Regions;

    let width = match name {
        "regular" => WideTile::WIDTH - 1,
        _ => WideTile::WIDTH,
    };

    let level = simd_level.to_level().unwrap_or_else(|| vello_cpu::Level::fallback());
    let simd_variant = simd_level.suffix();

    dispatch!(level, simd => {
        let fine = Fine::<_, U8Kernel>::new(simd);

        runner.run(
            &format!("fine/pack/{}", name),
            "fine/pack",
            name,
            simd_variant,
            || {
                let mut buf = vec![0; SCRATCH_BUF_SIZE];
                let mut regions = Regions::new(width, Tile::HEIGHT, &mut buf);
                regions.update_regions(|region| {
                    fine.pack(region);
                });
                std::hint::black_box(&regions);
            },
        )
    })
}

// ============================================================================
// Fine/Strip benchmark
// ============================================================================

fn run_fine_strip(runner: &BenchRunner, name: &str, simd_level: crate::SimdLevel) -> BenchmarkResult {
    use rand::prelude::StdRng;
    use rand::{Rng, SeedableRng};
    use vello_common::coarse::WideTile;
    use vello_common::color::palette::css::ROYAL_BLUE;
    use fearless_simd::dispatch;
    use vello_common::paint::{Paint, PremulColor};
    use vello_common::peniko::{BlendMode, Compose, Mix};
    use vello_common::tile::Tile;
    use vello_cpu::fine::{Fine, U8Kernel};

    const SEED: [u8; 32] = [0; 32];
    let blend = BlendMode::new(Mix::Normal, Compose::SrcOver);
    let paint = Paint::Solid(PremulColor::from_alpha_color(ROYAL_BLUE));

    let mut rng = StdRng::from_seed(SEED);
    let alphas: Vec<u8> = (0..WideTile::WIDTH as usize * Tile::HEIGHT as usize)
        .map(|_| rng.random())
        .collect();

    let width = match name {
        "solid_short" => 8,
        _ => 64,
    };

    let level = simd_level.to_level().unwrap_or_else(|| vello_cpu::Level::fallback());
    let simd_variant = simd_level.suffix();

    dispatch!(level, simd => {
        let mut fine = Fine::<_, U8Kernel>::new(simd);

        runner.run(
            &format!("fine/strip/{}", name),
            "fine/strip",
            name,
            simd_variant,
            || {
                fine.fill(0, width, &paint, blend, &[], Some(&alphas), None);
                std::hint::black_box(&fine);
            },
        )
    })
}

// ============================================================================
// Tile benchmark
// ============================================================================

fn run_tile(runner: &BenchRunner, name: &str, simd_level: crate::SimdLevel) -> Option<BenchmarkResult> {
    use vello_common::tile::Tiles;

    let items = get_data_items();
    let item = items.iter().find(|i| i.name == name)?;
    let lines = item.lines();

    let level = simd_level.to_level()?;
    let simd_variant = simd_level.suffix();

    Some(runner.run(
        &format!("tile/{}", name),
        "tile",
        name,
        simd_variant,
        || {
            let mut tiler = Tiles::new(level);
            tiler.make_tiles_analytic_aa(&lines, item.width, item.height);
            std::hint::black_box(&tiler);
        },
    ))
}

// ============================================================================
// Flatten benchmark
// ============================================================================

fn run_flatten(runner: &BenchRunner, name: &str, simd_level: crate::SimdLevel) -> Option<BenchmarkResult> {
    use vello_common::flatten::{self, FlattenCtx, Line};
    use vello_common::kurbo::Affine;

    let items = get_data_items();
    let item = items.iter().find(|i| i.name == name)?;
    let expanded_strokes = item.expanded_strokes();

    let level = simd_level.to_level()?;
    let simd_variant = simd_level.suffix();

    Some(runner.run(
        &format!("flatten/{}", name),
        "flatten",
        name,
        simd_variant,
        || {
            let mut line_buf: Vec<Line> = vec![];
            let mut temp_buf: Vec<Line> = vec![];
            let mut flatten_ctx = FlattenCtx::default();

            for path in &item.fills {
                flatten::fill(level, &path.path, path.transform, &mut temp_buf, &mut flatten_ctx);
                line_buf.extend(&temp_buf);
            }

            for stroke in &expanded_strokes {
                flatten::fill(level, stroke, Affine::IDENTITY, &mut temp_buf, &mut flatten_ctx);
                line_buf.extend(&temp_buf);
            }

            std::hint::black_box(&line_buf);
        },
    ))
}

// ============================================================================
// Strokes benchmark
// ============================================================================

fn run_strokes(runner: &BenchRunner, name: &str, simd_level: crate::SimdLevel) -> Option<BenchmarkResult> {
    use vello_common::flatten;
    use vello_common::kurbo::{Stroke, StrokeCtx};

    let items = get_data_items();
    let item = items.iter().find(|i| i.name == name)?;

    // strokes don't use SIMD level directly, but we still track the variant
    let simd_variant = simd_level.suffix();

    Some(runner.run(
        &format!("strokes/{}", name),
        "strokes",
        name,
        simd_variant,
        || {
            let mut stroke_ctx = StrokeCtx::default();
            let mut paths = vec![];

            for path in &item.strokes {
                let stroke = Stroke { width: path.stroke_width as f64, ..Default::default() };
                flatten::expand_stroke(path.path.iter(), &stroke, 0.25, &mut stroke_ctx);
                paths.push(stroke_ctx.output().clone());
            }

            std::hint::black_box(&paths);
        },
    ))
}

// ============================================================================
// Render strips benchmark
// ============================================================================

fn run_render_strips(runner: &BenchRunner, name: &str, simd_level: crate::SimdLevel) -> Option<BenchmarkResult> {
    use vello_common::peniko::Fill;
    use vello_common::strip::Strip;

    let items = get_data_items();
    let item = items.iter().find(|i| i.name == name)?;
    let lines = item.lines();
    let tiles = item.sorted_tiles();

    let level = simd_level.to_level()?;
    let simd_variant = simd_level.suffix();

    Some(runner.run(
        &format!("render_strips/{}", name),
        "render_strips",
        name,
        simd_variant,
        || {
            let mut strip_buf: Vec<Strip> = vec![];
            let mut alpha_buf: Vec<u8> = vec![];

            vello_common::strip::render(
                level,
                &tiles,
                &mut strip_buf,
                &mut alpha_buf,
                Fill::NonZero,
                None,
                &lines,
            );

            std::hint::black_box(&strip_buf);
        },
    ))
}

