// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::SEED;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use smallvec::{SmallVec, smallvec};
use vello_bench_macros::vello_bench;
use vello_common::coarse::WideTile;
use vello_common::color::palette::css::{BLUE, GREEN, RED, YELLOW};
use vello_common::color::{AlphaColor, DynamicColor, Srgb};
use vello_common::encode::EncodeExt;
use vello_common::fearless_simd::Simd;
use vello_common::kurbo::{Affine, Point};
use vello_common::peniko;
use vello_common::peniko::{BlendMode, ColorStop, ColorStops, Gradient, GradientKind};
use vello_common::tile::Tile;
use vello_cpu::fine::{Fine, FineKernel};
use vello_cpu::peniko::{LinearGradientPosition, RadialGradientPosition, SweepGradientPosition};

fn stops_opaque() -> ColorStops {
    ColorStops(smallvec![
        ColorStop {
            offset: 0.0,
            color: DynamicColor::from_alpha_color(BLUE),
        },
        ColorStop {
            offset: 0.33,
            color: DynamicColor::from_alpha_color(GREEN),
        },
        ColorStop {
            offset: 0.66,
            color: DynamicColor::from_alpha_color(RED),
        },
        ColorStop {
            offset: 1.0,
            color: DynamicColor::from_alpha_color(YELLOW),
        },
    ])
}

fn stops_transparent() -> ColorStops {
    ColorStops(smallvec![
        ColorStop {
            offset: 0.0,
            color: DynamicColor::from_alpha_color(BLUE),
        },
        ColorStop {
            offset: 0.33,
            color: DynamicColor::from_alpha_color(GREEN.with_alpha(0.5)),
        },
        ColorStop {
            offset: 0.66,
            color: DynamicColor::from_alpha_color(RED),
        },
        ColorStop {
            offset: 1.0,
            color: DynamicColor::from_alpha_color(YELLOW.with_alpha(0.7)),
        },
    ])
}

fn many_stops() -> ColorStops {
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

#[vello_bench]
fn linear_opaque<S: Simd, T: FineKernel<S>>(fine: &mut Fine<S, T>) {
    let kind: GradientKind = LinearGradientPosition {
        start: Point::new(128.0, 128.0),
        end: Point::new(134.0, 134.0),
    }
    .into();

    let mut paints = vec![];
    let grad = Gradient {
        kind,
        stops: stops_opaque(),
        extend: peniko::Extend::Pad,
        ..Default::default()
    };
    let paint = grad.encode_into(&mut paints, Affine::IDENTITY);

    fine.fill(
        0,
        WideTile::WIDTH as usize,
        &paint,
        BlendMode::default(),
        &paints,
        None,
        None,
    );
    std::hint::black_box(&fine);
}

#[vello_bench]
fn radial_opaque<S: Simd, T: FineKernel<S>>(fine: &mut Fine<S, T>) {
    let kind: GradientKind = RadialGradientPosition {
        start_center: Point::new(WideTile::WIDTH as f64 / 2.0, (Tile::HEIGHT / 2) as f64),
        start_radius: 25.0,
        end_center: Point::new(WideTile::WIDTH as f64 / 2.0, (Tile::HEIGHT / 2) as f64),
        end_radius: 75.0,
    }
    .into();

    let mut paints = vec![];
    let grad = Gradient {
        kind,
        stops: stops_opaque(),
        extend: peniko::Extend::Pad,
        ..Default::default()
    };
    let paint = grad.encode_into(&mut paints, Affine::IDENTITY);

    fine.fill(
        0,
        WideTile::WIDTH as usize,
        &paint,
        BlendMode::default(),
        &paints,
        None,
        None,
    );
    std::hint::black_box(&fine);
}

#[vello_bench]
fn sweep_opaque<S: Simd, T: FineKernel<S>>(fine: &mut Fine<S, T>) {
    let kind: GradientKind = SweepGradientPosition {
        center: Point::new(WideTile::WIDTH as f64 / 2.0, (Tile::HEIGHT / 2) as f64),
        start_angle: 70.0_f32.to_radians(),
        end_angle: 250.0_f32.to_radians(),
    }
    .into();

    let mut paints = vec![];
    let grad = Gradient {
        kind,
        stops: stops_opaque(),
        extend: peniko::Extend::Pad,
        ..Default::default()
    };
    let paint = grad.encode_into(&mut paints, Affine::IDENTITY);

    fine.fill(
        0,
        WideTile::WIDTH as usize,
        &paint,
        BlendMode::default(),
        &paints,
        None,
        None,
    );
    std::hint::black_box(&fine);
}

#[vello_bench]
fn gradient_many_stops<S: Simd, T: FineKernel<S>>(fine: &mut Fine<S, T>) {
    let kind: GradientKind = LinearGradientPosition {
        start: Point::new(128.0, 128.0),
        end: Point::new(134.0, 134.0),
    }
    .into();

    let mut paints = vec![];
    let grad = Gradient {
        kind,
        stops: many_stops(),
        extend: peniko::Extend::Repeat,
        ..Default::default()
    };
    let paint = grad.encode_into(&mut paints, Affine::IDENTITY);

    fine.fill(
        0,
        WideTile::WIDTH as usize,
        &paint,
        BlendMode::default(),
        &paints,
        None,
        None,
    );
    std::hint::black_box(&fine);
}

#[vello_bench]
fn gradient_transparent<S: Simd, T: FineKernel<S>>(fine: &mut Fine<S, T>) {
    let kind: GradientKind = LinearGradientPosition {
        start: Point::new(128.0, 128.0),
        end: Point::new(134.0, 134.0),
    }
    .into();

    let mut paints = vec![];
    let grad = Gradient {
        kind,
        stops: stops_transparent(),
        extend: peniko::Extend::Pad,
        ..Default::default()
    };
    let paint = grad.encode_into(&mut paints, Affine::IDENTITY);

    fine.fill(
        0,
        WideTile::WIDTH as usize,
        &paint,
        BlendMode::default(),
        &paints,
        None,
        None,
    );
    std::hint::black_box(&fine);
}

pub fn run_benchmarks() {
    linear_opaque();
    radial_opaque();
    sweep_opaque();
    gradient_many_stops();
    gradient_transparent();
}
