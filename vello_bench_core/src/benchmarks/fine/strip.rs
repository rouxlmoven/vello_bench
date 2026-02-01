// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::benchmarks::SEED;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use vello_bench_macros::vello_bench;
use vello_common::coarse::WideTile;
use vello_common::color::palette::css::ROYAL_BLUE;
use vello_common::fearless_simd::Simd;
use vello_common::paint::{Paint, PremulColor};
use vello_common::peniko::BlendMode;
use vello_common::tile::Tile;
use vello_cpu::fine::{Fine, FineKernel};

fn get_alphas() -> Vec<u8> {
    let mut rng = StdRng::from_seed(SEED);
    (0..WideTile::WIDTH as usize * Tile::HEIGHT as usize)
        .map(|_| rng.random())
        .collect()
}

#[vello_bench]
fn solid_short<S: Simd, T: FineKernel<S>>(fine: &mut Fine<S, T>) {
    let paint = Paint::Solid(PremulColor::from_alpha_color(ROYAL_BLUE));
    let alphas = get_alphas();
    fine.fill(0, 8, &paint, BlendMode::default(), &[], Some(&alphas), None);
    std::hint::black_box(&fine);
}

#[vello_bench]
fn solid_long<S: Simd, T: FineKernel<S>>(fine: &mut Fine<S, T>) {
    let paint = Paint::Solid(PremulColor::from_alpha_color(ROYAL_BLUE));
    let alphas = get_alphas();
    fine.fill(0, 64, &paint, BlendMode::default(), &[], Some(&alphas), None);
    std::hint::black_box(&fine);
}

pub fn run_benchmarks() {
    solid_short();
    solid_long();
}
