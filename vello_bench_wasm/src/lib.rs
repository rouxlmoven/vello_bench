// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! WASM bindings for vello benchmarks.

#![allow(missing_docs, reason = "Not needed for benchmarks")]

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use vello_bench_core::{BenchRunner, BenchmarkResult, PlatformInfo, SimdLevel};

/// Initialize the WASM module.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Benchmark info for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkInfo {
    pub id: String,
    pub category: String,
    pub name: String,
    pub simd_variant: String,
}

/// List all available benchmarks.
#[wasm_bindgen]
pub fn list_benchmarks() -> JsValue {
    let mut benchmarks = vec![
        // Fine/Fill benchmarks
        BenchmarkInfo {
            id: "fine/fill/opaque_short".into(),
            category: "fine/fill".into(),
            name: "opaque_short".into(),
            simd_variant: "wasm".into(),
        },
        BenchmarkInfo {
            id: "fine/fill/opaque_long".into(),
            category: "fine/fill".into(),
            name: "opaque_long".into(),
            simd_variant: "wasm".into(),
        },
        BenchmarkInfo {
            id: "fine/fill/transparent_short".into(),
            category: "fine/fill".into(),
            name: "transparent_short".into(),
            simd_variant: "wasm".into(),
        },
        BenchmarkInfo {
            id: "fine/fill/transparent_long".into(),
            category: "fine/fill".into(),
            name: "transparent_long".into(),
            simd_variant: "wasm".into(),
        },
        // Fine/Gradient benchmarks
        BenchmarkInfo {
            id: "fine/gradient/linear_opaque".into(),
            category: "fine/gradient".into(),
            name: "linear_opaque".into(),
            simd_variant: "wasm".into(),
        },
        BenchmarkInfo {
            id: "fine/gradient/radial_opaque".into(),
            category: "fine/gradient".into(),
            name: "radial_opaque".into(),
            simd_variant: "wasm".into(),
        },
        BenchmarkInfo {
            id: "fine/gradient/sweep_opaque".into(),
            category: "fine/gradient".into(),
            name: "sweep_opaque".into(),
            simd_variant: "wasm".into(),
        },
        BenchmarkInfo {
            id: "fine/gradient/many_stops".into(),
            category: "fine/gradient".into(),
            name: "many_stops".into(),
            simd_variant: "wasm".into(),
        },
        BenchmarkInfo {
            id: "fine/gradient/transparent".into(),
            category: "fine/gradient".into(),
            name: "transparent".into(),
            simd_variant: "wasm".into(),
        },
        // Fine/Image benchmarks
        BenchmarkInfo {
            id: "fine/image/no_transform".into(),
            category: "fine/image".into(),
            name: "no_transform".into(),
            simd_variant: "wasm".into(),
        },
        BenchmarkInfo {
            id: "fine/image/scale".into(),
            category: "fine/image".into(),
            name: "scale".into(),
            simd_variant: "wasm".into(),
        },
        BenchmarkInfo {
            id: "fine/image/rotate".into(),
            category: "fine/image".into(),
            name: "rotate".into(),
            simd_variant: "wasm".into(),
        },
        BenchmarkInfo {
            id: "fine/image/quality_medium".into(),
            category: "fine/image".into(),
            name: "quality_medium".into(),
            simd_variant: "wasm".into(),
        },
        BenchmarkInfo {
            id: "fine/image/quality_high".into(),
            category: "fine/image".into(),
            name: "quality_high".into(),
            simd_variant: "wasm".into(),
        },
        BenchmarkInfo {
            id: "fine/image/extend_repeat".into(),
            category: "fine/image".into(),
            name: "extend_repeat".into(),
            simd_variant: "wasm".into(),
        },
        // Fine/Pack benchmarks
        BenchmarkInfo {
            id: "fine/pack/block".into(),
            category: "fine/pack".into(),
            name: "block".into(),
            simd_variant: "wasm".into(),
        },
        BenchmarkInfo {
            id: "fine/pack/regular".into(),
            category: "fine/pack".into(),
            name: "regular".into(),
            simd_variant: "wasm".into(),
        },
        // Fine/Strip benchmarks
        BenchmarkInfo {
            id: "fine/strip/solid_short".into(),
            category: "fine/strip".into(),
            name: "solid_short".into(),
            simd_variant: "wasm".into(),
        },
        BenchmarkInfo {
            id: "fine/strip/solid_long".into(),
            category: "fine/strip".into(),
            name: "solid_long".into(),
            simd_variant: "wasm".into(),
        },
    ];

    // Add SVG-based benchmarks (tile, flatten, render_strips)
    // These use embedded Ghostscript Tiger SVG data
    let data_items = vello_bench_core::data::get_data_items();
    for item in data_items {
        // Tile benchmarks
        benchmarks.push(BenchmarkInfo {
            id: format!("tile/{}", item.name),
            category: "tile".into(),
            name: item.name.clone(),
            simd_variant: "wasm".into(),
        });
        // Flatten benchmarks
        benchmarks.push(BenchmarkInfo {
            id: format!("flatten/{}", item.name),
            category: "flatten".into(),
            name: item.name.clone(),
            simd_variant: "wasm".into(),
        });
        // Stroke expansion benchmarks
        benchmarks.push(BenchmarkInfo {
            id: format!("strokes/{}", item.name),
            category: "strokes".into(),
            name: item.name.clone(),
            simd_variant: "wasm".into(),
        });
        // Render strips benchmarks
        benchmarks.push(BenchmarkInfo {
            id: format!("render_strips/{}", item.name),
            category: "render_strips".into(),
            name: item.name.clone(),
            simd_variant: "wasm".into(),
        });
    }

    serde_wasm_bindgen::to_value(&benchmarks).unwrap()
}

/// Get available SIMD levels for this platform.
#[wasm_bindgen]
pub fn get_simd_levels() -> JsValue {
    let levels = SimdLevel::available();
    let level_info: Vec<_> = levels
        .into_iter()
        .map(|l| serde_json::json!({
            "id": l.suffix(),
            "name": l.display_name(),
        }))
        .collect();
    serde_wasm_bindgen::to_value(&level_info).unwrap()
}

/// Check if SIMD128 is available.
#[wasm_bindgen]
pub fn has_simd128() -> bool {
    #[cfg(target_feature = "simd128")]
    {
        true
    }
    #[cfg(not(target_feature = "simd128"))]
    {
        false
    }
}

/// Run a single benchmark by ID.
#[wasm_bindgen]
pub fn run_benchmark(id: &str, warmup_ms: u64, measurement_ms: u64) -> JsValue {
    let runner = BenchRunner::new(warmup_ms, measurement_ms);

    let result = match id {
        // Fine/Fill benchmarks
        "fine/fill/opaque_short" => Some(run_fine_fill_benchmark(&runner, "opaque_short")),
        "fine/fill/opaque_long" => Some(run_fine_fill_benchmark(&runner, "opaque_long")),
        "fine/fill/transparent_short" => Some(run_fine_fill_benchmark(&runner, "transparent_short")),
        "fine/fill/transparent_long" => Some(run_fine_fill_benchmark(&runner, "transparent_long")),
        // Fine/Gradient benchmarks
        "fine/gradient/linear_opaque" => Some(run_fine_gradient_benchmark(&runner, "linear_opaque")),
        "fine/gradient/radial_opaque" => Some(run_fine_gradient_benchmark(&runner, "radial_opaque")),
        "fine/gradient/sweep_opaque" => Some(run_fine_gradient_benchmark(&runner, "sweep_opaque")),
        "fine/gradient/many_stops" => Some(run_fine_gradient_benchmark(&runner, "many_stops")),
        "fine/gradient/transparent" => Some(run_fine_gradient_benchmark(&runner, "transparent")),
        // Fine/Image benchmarks
        "fine/image/no_transform" => Some(run_fine_image_benchmark(&runner, "no_transform")),
        "fine/image/scale" => Some(run_fine_image_benchmark(&runner, "scale")),
        "fine/image/rotate" => Some(run_fine_image_benchmark(&runner, "rotate")),
        "fine/image/quality_medium" => Some(run_fine_image_benchmark(&runner, "quality_medium")),
        "fine/image/quality_high" => Some(run_fine_image_benchmark(&runner, "quality_high")),
        "fine/image/extend_repeat" => Some(run_fine_image_benchmark(&runner, "extend_repeat")),
        // Fine/Pack benchmarks
        "fine/pack/block" => Some(run_fine_pack_benchmark(&runner, "block")),
        "fine/pack/regular" => Some(run_fine_pack_benchmark(&runner, "regular")),
        // Fine/Strip benchmarks
        "fine/strip/solid_short" => Some(run_fine_strip_benchmark(&runner, "solid_short")),
        "fine/strip/solid_long" => Some(run_fine_strip_benchmark(&runner, "solid_long")),
        // SVG-based benchmarks (tile, flatten, strokes, render_strips)
        id if id.starts_with("tile/") => {
            let name = &id["tile/".len()..];
            run_tile_benchmark(&runner, name)
        }
        id if id.starts_with("flatten/") => {
            let name = &id["flatten/".len()..];
            run_flatten_benchmark(&runner, name)
        }
        id if id.starts_with("strokes/") => {
            let name = &id["strokes/".len()..];
            run_strokes_benchmark(&runner, name)
        }
        id if id.starts_with("render_strips/") => {
            let name = &id["render_strips/".len()..];
            run_render_strips_benchmark(&runner, name)
        }
        _ => None,
    };

    match result {
        Some(r) => serde_wasm_bindgen::to_value(&r).unwrap(),
        None => JsValue::NULL,
    }
}

/// Get platform information.
#[wasm_bindgen]
pub fn get_platform_info() -> JsValue {
    let info = PlatformInfo::detect();
    serde_wasm_bindgen::to_value(&info).unwrap()
}

// Benchmark implementations for WASM

fn run_fine_fill_benchmark(runner: &BenchRunner, name: &str) -> BenchmarkResult {
    use vello_common::color::palette::css::ROYAL_BLUE;
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

    #[cfg(target_feature = "simd128")]
    let simd_variant = "wasm_simd128";
    #[cfg(not(target_feature = "simd128"))]
    let simd_variant = "wasm_scalar";

    // WASM uses scalar or simd128 depending on build
    let mut fine = Fine::<_, U8Kernel>::new(vello_common::fearless_simd::Fallback::new());

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
}

fn run_fine_gradient_benchmark(runner: &BenchRunner, name: &str) -> BenchmarkResult {
    use rand::prelude::StdRng;
    use rand::{Rng, SeedableRng};
    use smallvec::{SmallVec, smallvec};
    use vello_common::coarse::WideTile;
    use vello_common::color::palette::css::{BLUE, GREEN, RED, YELLOW};
    use vello_common::color::{AlphaColor, DynamicColor, Srgb};
    use vello_common::encode::EncodeExt;
    use vello_common::kurbo::{Affine, Point};
    use vello_common::peniko::{BlendMode, ColorStop, ColorStops, Compose, Gradient, GradientKind, Mix};
    use vello_common::tile::Tile;
    use vello_cpu::fine::{Fine, U8Kernel};
    use vello_cpu::peniko::{LinearGradientPosition, RadialGradientPosition, SweepGradientPosition};
    let blend = BlendMode::new(Mix::Normal, Compose::SrcOver);

    const SEED: [u8; 32] = [0; 32];

    // Get stops based on benchmark name
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

    let grad = Gradient {
        kind,
        stops,
        extend,
        ..Default::default()
    };

    #[cfg(target_feature = "simd128")]
    let simd_variant = "wasm_simd128";
    #[cfg(not(target_feature = "simd128"))]
    let simd_variant = "wasm_scalar";

    let mut fine = Fine::<_, U8Kernel>::new(vello_common::fearless_simd::Fallback::new());
    let mut paints = vec![];
    let paint = grad.encode_into(&mut paints, Affine::IDENTITY);

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
}

fn run_fine_image_benchmark(runner: &BenchRunner, name: &str) -> BenchmarkResult {
    use std::sync::Arc;
    use vello_common::coarse::WideTile;
    use vello_common::encode::EncodeExt;
    use vello_common::kurbo::{Affine, Point};
    use vello_common::paint::{Image, ImageSource};
    use vello_common::peniko::{BlendMode, Compose, Extend, ImageQuality, ImageSampler, Mix};
    use vello_common::pixmap::Pixmap;
    use vello_common::tile::Tile;
    use vello_cpu::fine::{Fine, U8Kernel};
    let blend = BlendMode::new(Mix::Normal, Compose::SrcOver);

    // Determine quality based on benchmark name
    let quality = match name {
        "quality_medium" => ImageQuality::Medium,
        "quality_high" => ImageQuality::High,
        _ => ImageQuality::Low,
    };

    // Determine extend mode
    let extend = match name {
        "extend_repeat" => Extend::Repeat,
        _ => Extend::Pad,
    };

    // Use different images for extend tests
    let (data, transform): (&[u8], Affine) = match name {
        "extend_repeat" => {
            let small_data = include_bytes!("../../vello_bench_core/assets/rgb_image_2x2.png");
            (small_data, Affine::translate((WideTile::WIDTH as f64 / 2.0, 0.0)))
        }
        "scale" | "quality_medium" | "quality_high" => {
            let colr_data = include_bytes!("../../vello_bench_core/assets/big_colr.png");
            (colr_data, Affine::scale(3.0))
        }
        "rotate" => {
            let colr_data = include_bytes!("../../vello_bench_core/assets/big_colr.png");
            (colr_data, Affine::rotate_about(
                1.0,
                Point::new(WideTile::WIDTH as f64 / 2.0, Tile::HEIGHT as f64 / 2.0),
            ))
        }
        _ => {
            let colr_data = include_bytes!("../../vello_bench_core/assets/big_colr.png");
            (colr_data, Affine::IDENTITY)
        }
    };

    let pixmap = Pixmap::from_png(data).unwrap();
    let image = Image {
        image: ImageSource::Pixmap(Arc::new(pixmap)),
        sampler: ImageSampler {
            x_extend: extend,
            y_extend: extend,
            quality,
            alpha: 1.0,
        },
    };

    #[cfg(target_feature = "simd128")]
    let simd_variant = "wasm_simd128";
    #[cfg(not(target_feature = "simd128"))]
    let simd_variant = "wasm_scalar";

    let mut fine = Fine::<_, U8Kernel>::new(vello_common::fearless_simd::Fallback::new());
    let mut paints = vec![];
    let paint = image.encode_into(&mut paints, transform);

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
}

fn run_fine_pack_benchmark(runner: &BenchRunner, name: &str) -> BenchmarkResult {
    use vello_common::coarse::WideTile;
    use vello_common::tile::Tile;
    use vello_cpu::fine::{Fine, U8Kernel, SCRATCH_BUF_SIZE};
    use vello_cpu::region::Regions;

    #[cfg(target_feature = "simd128")]
    let simd_variant = "wasm_simd128";
    #[cfg(not(target_feature = "simd128"))]
    let simd_variant = "wasm_scalar";

    let fine = Fine::<_, U8Kernel>::new(vello_common::fearless_simd::Fallback::new());

    let width = match name {
        "regular" => WideTile::WIDTH - 1,
        _ => WideTile::WIDTH,
    };

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
}

fn run_fine_strip_benchmark(runner: &BenchRunner, name: &str) -> BenchmarkResult {
    use rand::prelude::StdRng;
    use rand::{Rng, SeedableRng};
    use vello_common::coarse::WideTile;
    use vello_common::color::palette::css::ROYAL_BLUE;
    use vello_common::paint::{Paint, PremulColor};
    use vello_common::peniko::{BlendMode, Compose, Mix};
    use vello_common::tile::Tile;
    use vello_cpu::fine::{Fine, U8Kernel};

    const SEED: [u8; 32] = [0; 32];
    let blend = BlendMode::new(Mix::Normal, Compose::SrcOver);
    let paint = Paint::Solid(PremulColor::from_alpha_color(ROYAL_BLUE));

    // Generate random alpha values
    let mut rng = StdRng::from_seed(SEED);
    let alphas: Vec<u8> = (0..WideTile::WIDTH as usize * Tile::HEIGHT as usize)
        .map(|_| rng.random())
        .collect();

    let width = match name {
        "solid_short" => 8,
        _ => 64,
    };

    #[cfg(target_feature = "simd128")]
    let simd_variant = "wasm_simd128";
    #[cfg(not(target_feature = "simd128"))]
    let simd_variant = "wasm_scalar";

    let mut fine = Fine::<_, U8Kernel>::new(vello_common::fearless_simd::Fallback::new());

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
}

// SVG-based benchmarks using embedded data

fn run_tile_benchmark(runner: &BenchRunner, name: &str) -> Option<BenchmarkResult> {
    use vello_bench_core::data::get_data_items;
    use vello_common::tile::Tiles;
    use vello_cpu::Level;

    let data_items = get_data_items();
    let item = data_items.iter().find(|i| i.name == name)?;

    let lines = item.lines();

    #[cfg(target_feature = "simd128")]
    let simd_variant = "wasm_simd128";
    #[cfg(not(target_feature = "simd128"))]
    let simd_variant = "wasm_scalar";

    Some(runner.run(
        &format!("tile/{}", name),
        "tile",
        name,
        simd_variant,
        || {
            let mut tiler = Tiles::new(Level::new());
            tiler.make_tiles_analytic_aa(&lines, item.width, item.height);
            std::hint::black_box(&tiler);
        },
    ))
}

fn run_flatten_benchmark(runner: &BenchRunner, name: &str) -> Option<BenchmarkResult> {
    use vello_bench_core::data::get_data_items;
    use vello_common::flatten;
    use vello_common::flatten::FlattenCtx;
    use vello_common::kurbo::Affine;
    use vello_cpu::Level;

    let data_items = get_data_items();
    let item = data_items.iter().find(|i| i.name == name)?;

    let expanded_strokes = item.expanded_strokes();

    #[cfg(target_feature = "simd128")]
    let simd_variant = "wasm_simd128";
    #[cfg(not(target_feature = "simd128"))]
    let simd_variant = "wasm_scalar";

    Some(runner.run(
        &format!("flatten/{}", name),
        "flatten",
        name,
        simd_variant,
        || {
            let mut line_buf: Vec<flatten::Line> = vec![];
            let mut temp_buf: Vec<flatten::Line> = vec![];
            let mut flatten_ctx = FlattenCtx::default();

            line_buf.clear();

            for path in &item.fills {
                flatten::fill(
                    Level::new(),
                    &path.path,
                    path.transform,
                    &mut temp_buf,
                    &mut flatten_ctx,
                );
                line_buf.extend(&temp_buf);
            }

            for stroke in &expanded_strokes {
                flatten::fill(
                    Level::new(),
                    stroke,
                    Affine::IDENTITY,
                    &mut temp_buf,
                    &mut flatten_ctx,
                );
                line_buf.extend(&temp_buf);
            }

            std::hint::black_box(&line_buf);
        },
    ))
}

fn run_strokes_benchmark(runner: &BenchRunner, name: &str) -> Option<BenchmarkResult> {
    use vello_bench_core::data::get_data_items;
    use vello_common::flatten;
    use vello_common::kurbo::{Stroke, StrokeCtx};

    let data_items = get_data_items();
    let item = data_items.iter().find(|i| i.name == name)?;

    #[cfg(target_feature = "simd128")]
    let simd_variant = "wasm_simd128";
    #[cfg(not(target_feature = "simd128"))]
    let simd_variant = "wasm_scalar";

    Some(runner.run(
        &format!("strokes/{}", name),
        "strokes",
        name,
        simd_variant,
        || {
            let mut stroke_ctx = StrokeCtx::default();
            let mut paths = vec![];

            for path in &item.strokes {
                let stroke = Stroke {
                    width: path.stroke_width as f64,
                    ..Default::default()
                };
                flatten::expand_stroke(path.path.iter(), &stroke, 0.25, &mut stroke_ctx);
                paths.push(stroke_ctx.output().clone());
            }

            std::hint::black_box(&paths);
        },
    ))
}

fn run_render_strips_benchmark(runner: &BenchRunner, name: &str) -> Option<BenchmarkResult> {
    use vello_bench_core::data::get_data_items;
    use vello_common::fearless_simd::Level;
    use vello_common::peniko::Fill;

    let data_items = get_data_items();
    let item = data_items.iter().find(|i| i.name == name)?;

    let lines = item.lines();
    let tiles = item.sorted_tiles();
    let simd_level = Level::new();

    #[cfg(target_feature = "simd128")]
    let simd_variant = "wasm_simd128";
    #[cfg(not(target_feature = "simd128"))]
    let simd_variant = "wasm_scalar";

    Some(runner.run(
        &format!("render_strips/{}", name),
        "render_strips",
        name,
        simd_variant,
        || {
            let mut strip_buf = vec![];
            let mut alpha_buf = vec![];

            strip_buf.clear();
            alpha_buf.clear();

            vello_common::strip::render(
                simd_level,
                &tiles,
                &mut strip_buf,
                &mut alpha_buf,
                Fill::NonZero,
                None,
                &lines,
            );
            std::hint::black_box((&strip_buf, &alpha_buf));
        },
    ))
}
