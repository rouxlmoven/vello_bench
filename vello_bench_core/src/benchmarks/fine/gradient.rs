// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::registry::BenchmarkInfo;
use crate::result::BenchmarkResult;
use crate::runner::BenchRunner;
use crate::simd::level_suffix;
use fearless_simd::{Level, dispatch};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use smallvec::{SmallVec, smallvec};
use vello_common::coarse::WideTile;
use vello_common::color::palette::css::{BLUE, GREEN, RED, YELLOW};
use vello_common::color::{AlphaColor, DynamicColor, Srgb};
use vello_common::encode::EncodeExt;
use vello_common::kurbo::{Affine, Point};
use vello_common::peniko::{
    BlendMode, ColorStop, ColorStops, Compose, Gradient, GradientKind, Mix,
};
use vello_common::tile::Tile;
use vello_cpu::fine::{Fine, U8Kernel};
use vello_cpu::peniko::{LinearGradientPosition, RadialGradientPosition, SweepGradientPosition};

const NAMES: &[&str] = &["linear_opaque", "radial_opaque", "sweep_opaque", "many_stops", "transparent"];
const CATEGORY: &str = "fine/gradient";
const SEED: [u8; 32] = [0; 32];

pub fn list() -> Vec<BenchmarkInfo> {
    BenchmarkInfo::from_names(CATEGORY, NAMES)
}

pub fn run(name: &str, runner: &BenchRunner, level: Level) -> Option<BenchmarkResult> {
    if !NAMES.contains(&name) {
        return None;
    }

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
        }
        .into(),
        "sweep_opaque" => SweepGradientPosition {
            center: Point::new(WideTile::WIDTH as f64 / 2.0, (Tile::HEIGHT / 2) as f64),
            start_angle: 70.0_f32.to_radians(),
            end_angle: 250.0_f32.to_radians(),
        }
        .into(),
        _ => LinearGradientPosition {
            start: Point::new(128.0, 128.0),
            end: Point::new(134.0, 134.0),
        }
        .into(),
    };

    let extend = match name {
        "many_stops" => vello_common::peniko::Extend::Repeat,
        _ => vello_common::peniko::Extend::Pad,
    };

    let grad = Gradient { kind, stops, extend, ..Default::default() };
    let mut paints = vec![];
    let paint = grad.encode_into(&mut paints, Affine::IDENTITY);

    let simd_variant = level_suffix(level);

    Some(dispatch!(level, simd => {
        let mut fine = Fine::<_, U8Kernel>::new(simd);

        runner.run(
            &format!("{CATEGORY}/{name}"),
            CATEGORY,
            name,
            simd_variant,
            #[inline(always)]
            || {
                fine.fill(0, WideTile::WIDTH as usize, &paint, blend, &paints, None, None);
                std::hint::black_box(&fine);
            },
        )
    }))
}
