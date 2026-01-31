// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::fine::default_blend;
use vello_bench_macros::vello_bench;
use vello_common::blurred_rounded_rect::BlurredRoundedRectangle;
use vello_common::coarse::WideTile;
use vello_common::color::palette::css::GREEN;
use vello_common::encode::EncodeExt;
use vello_common::fearless_simd::Simd;
use vello_common::kurbo::{Affine, Point, Rect};
use vello_common::tile::Tile;
use vello_cpu::fine::{Fine, FineKernel};

#[vello_bench]
fn no_transform<S: Simd, T: FineKernel<S>>(fine: &mut Fine<S, T>) {
    let mut paints = vec![];
    let rect = BlurredRoundedRectangle {
        rect: Rect::new(0.0, 0.0, WideTile::WIDTH as f64, Tile::HEIGHT as f64),
        color: GREEN,
        radius: 30.0,
        std_dev: 10.0,
    };
    let paint = rect.encode_into(&mut paints, Affine::IDENTITY);

    fine.fill(
        0,
        WideTile::WIDTH as usize,
        &paint,
        default_blend(),
        &paints,
        None,
        None,
    );
    std::hint::black_box(&fine);
}

#[vello_bench]
fn with_transform<S: Simd, T: FineKernel<S>>(fine: &mut Fine<S, T>) {
    let center = Point::new(WideTile::WIDTH as f64 / 2.0, Tile::HEIGHT as f64 / 2.0);
    let mut paints = vec![];
    let rect = BlurredRoundedRectangle {
        rect: Rect::new(0.0, 0.0, WideTile::WIDTH as f64, Tile::HEIGHT as f64),
        color: GREEN,
        radius: 30.0,
        std_dev: 10.0,
    };
    let paint = rect.encode_into(&mut paints, Affine::rotate_about(1.0, center));

    fine.fill(
        0,
        WideTile::WIDTH as usize,
        &paint,
        default_blend(),
        &paints,
        None,
        None,
    );
    std::hint::black_box(&fine);
}

pub fn run_benchmarks() {
    no_transform();
    with_transform();
}
