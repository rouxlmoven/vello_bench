// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::registry::BenchmarkInfo;
use crate::result::BenchmarkResult;
use crate::runner::BenchRunner;
use crate::simd::level_suffix;
use fearless_simd::{Level, dispatch};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use vello_common::coarse::WideTile;
use vello_common::color::palette::css::ROYAL_BLUE;
use vello_common::paint::{Paint, PremulColor};
use vello_common::peniko::{BlendMode, Compose, Mix};
use vello_common::tile::Tile;
use vello_cpu::fine::{Fine, U8Kernel};

const NAMES: &[&str] = &["solid_short", "solid_long"];
const CATEGORY: &str = "fine/strip";
const SEED: [u8; 32] = [0; 32];

pub fn list() -> Vec<BenchmarkInfo> {
    NAMES
        .iter()
        .map(|name| BenchmarkInfo {
            id: format!("{CATEGORY}/{name}"),
            category: CATEGORY.into(),
            name: (*name).into(),
        })
        .collect()
}

pub fn run(name: &str, runner: &BenchRunner, level: Level) -> Option<BenchmarkResult> {
    if !NAMES.contains(&name) {
        return None;
    }

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

    let simd_variant = level_suffix(level);

    Some(dispatch!(level, simd => {
        let mut fine = Fine::<_, U8Kernel>::new(simd);

        runner.run(
            &format!("{CATEGORY}/{name}"),
            CATEGORY,
            name,
            simd_variant,
            || {
                fine.fill(0, width, &paint, blend, &[], Some(&alphas), None);
                std::hint::black_box(&fine);
            },
        )
    }))
}
