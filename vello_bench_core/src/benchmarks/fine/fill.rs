// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::registry::BenchmarkInfo;
use crate::result::BenchmarkResult;
use crate::runner::BenchRunner;
use crate::simd::level_suffix;
use fearless_simd::{Level, dispatch};
use vello_common::color::palette::css::ROYAL_BLUE;
use vello_common::paint::{Paint, PremulColor};
use vello_common::peniko::{BlendMode, Compose, Mix};
use vello_cpu::fine::{Fine, U8Kernel};

const NAMES: &[&str] = &["opaque_short", "opaque_long", "transparent_short", "transparent_long"];
const CATEGORY: &str = "fine/fill";

pub fn list() -> Vec<BenchmarkInfo> {
    BenchmarkInfo::from_names(CATEGORY, NAMES)
}

pub fn run(name: &str, runner: &BenchRunner, level: Level) -> Option<BenchmarkResult> {
    if !NAMES.contains(&name) {
        return None;
    }

    let blend = BlendMode::new(Mix::Normal, Compose::SrcOver);

    let width = match name {
        "opaque_short" | "transparent_short" => 32,
        _ => 256,
    };

    let alpha = if name.contains("transparent") { 0.3 } else { 1.0 };
    let paint = Paint::Solid(PremulColor::from_alpha_color(ROYAL_BLUE.with_alpha(alpha)));

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
                fine.fill(0, width, &paint, blend, &[], None, None);
                std::hint::black_box(&fine);
            },
        )
    }))
}
