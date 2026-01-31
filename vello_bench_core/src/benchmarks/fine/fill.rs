// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::benchmarks::fine::default_blend;
use vello_bench_macros::vello_bench;
use vello_common::color::palette::css::ROYAL_BLUE;
use vello_common::fearless_simd::Simd;
use vello_common::paint::{Paint, PremulColor};
use vello_cpu::fine::{Fine, FineKernel};

#[vello_bench]
fn opaque_short<S: Simd, T: FineKernel<S>>(fine: &mut Fine<S, T>) {
    let paint = Paint::Solid(PremulColor::from_alpha_color(ROYAL_BLUE));
    fine.fill(0, 32, &paint, default_blend(), &[], None, None);
    std::hint::black_box(&fine);
}

#[vello_bench]
fn opaque_long<S: Simd, T: FineKernel<S>>(fine: &mut Fine<S, T>) {
    let paint = Paint::Solid(PremulColor::from_alpha_color(ROYAL_BLUE));
    fine.fill(0, 256, &paint, default_blend(), &[], None, None);
    std::hint::black_box(&fine);
}

#[vello_bench]
fn transparent_short<S: Simd, T: FineKernel<S>>(fine: &mut Fine<S, T>) {
    let paint = Paint::Solid(PremulColor::from_alpha_color(ROYAL_BLUE.with_alpha(0.3)));
    fine.fill(0, 32, &paint, default_blend(), &[], None, None);
    std::hint::black_box(&fine);
}

#[vello_bench]
fn transparent_long<S: Simd, T: FineKernel<S>>(fine: &mut Fine<S, T>) {
    let paint = Paint::Solid(PremulColor::from_alpha_color(ROYAL_BLUE.with_alpha(0.3)));
    fine.fill(0, 256, &paint, default_blend(), &[], None, None);
    std::hint::black_box(&fine);
}

pub fn run_benchmarks() {
    opaque_short();
    opaque_long();
    transparent_short();
    transparent_long();
}
