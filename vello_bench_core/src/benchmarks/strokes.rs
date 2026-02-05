// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::data::get_data_items;
use crate::registry::BenchmarkInfo;
use crate::result::BenchmarkResult;
use crate::runner::BenchRunner;
use crate::simd::level_suffix;
use fearless_simd::Level;
use vello_common::flatten;
use vello_common::kurbo::{Stroke, StrokeCtx};

const CATEGORY: &str = "strokes";

pub fn list() -> Vec<BenchmarkInfo> {
    BenchmarkInfo::from_data_items(CATEGORY)
}

pub fn run(name: &str, runner: &BenchRunner, level: Level) -> Option<BenchmarkResult> {
    let items = get_data_items();
    let item = items.iter().find(|i| i.name == name)?;
    let simd_variant = level_suffix(level);

    // strokes don't use SIMD level directly, but we still track the variant
    let _ = level;

    Some(runner.run(
        &format!("{CATEGORY}/{name}"),
        CATEGORY,
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
