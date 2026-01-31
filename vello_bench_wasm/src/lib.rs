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
    let benchmarks = vec![
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
            id: "fine/pack/block".into(),
            category: "fine/pack".into(),
            name: "block".into(),
            simd_variant: "wasm".into(),
        },
        BenchmarkInfo {
            id: "integration/images_overlapping".into(),
            category: "integration".into(),
            name: "images_overlapping".into(),
            simd_variant: "wasm".into(),
        },
    ];
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
        "fine/fill/opaque_short" => Some(run_fine_fill_benchmark(&runner, "opaque_short")),
        "fine/fill/opaque_long" => Some(run_fine_fill_benchmark(&runner, "opaque_long")),
        "fine/fill/transparent_short" => Some(run_fine_fill_benchmark(&runner, "transparent_short")),
        "fine/fill/transparent_long" => Some(run_fine_fill_benchmark(&runner, "transparent_long")),
        "fine/gradient/linear_opaque" => Some(run_fine_gradient_benchmark(&runner, "linear_opaque")),
        "fine/gradient/radial_opaque" => Some(run_fine_gradient_benchmark(&runner, "radial_opaque")),
        "fine/image/no_transform" => Some(run_fine_image_benchmark(&runner, "no_transform")),
        "fine/image/scale" => Some(run_fine_image_benchmark(&runner, "scale")),
        "fine/pack/block" => Some(run_fine_pack_benchmark(&runner, "block")),
        "integration/images_overlapping" => Some(run_integration_benchmark(&runner, "images_overlapping")),
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
    use smallvec::smallvec;
    use vello_common::coarse::WideTile;
    use vello_common::color::palette::css::{BLUE, GREEN, RED, YELLOW};
    use vello_common::color::DynamicColor;
    use vello_common::encode::EncodeExt;
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
    use vello_common::kurbo::Affine;
    use vello_common::paint::{Image, ImageSource};
    use vello_common::peniko::{BlendMode, Compose, Extend, ImageQuality, ImageSampler, Mix};
    use vello_common::pixmap::Pixmap;
    use vello_cpu::fine::{Fine, U8Kernel};
    let blend = BlendMode::new(Mix::Normal, Compose::SrcOver);

    // Use embedded image data
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

    runner.run(
        &format!("fine/pack/{}", name),
        "fine/pack",
        name,
        simd_variant,
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

    #[cfg(target_feature = "simd128")]
    let simd_variant = "wasm_simd128";
    #[cfg(not(target_feature = "simd128"))]
    let simd_variant = "wasm_scalar";

    runner.run(
        &format!("integration/{}", name),
        "integration",
        name,
        simd_variant,
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
