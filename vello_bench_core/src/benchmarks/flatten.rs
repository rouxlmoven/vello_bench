// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::data::get_data_items;
use crate::registry::BenchmarkInfo;
use crate::result::BenchmarkResult;
use crate::runner::BenchRunner;
use crate::simd::level_suffix;
use fearless_simd::Level;
use vello_common::flatten::{self, FlattenCtx, Line};
use vello_common::kurbo::Affine;

const CATEGORY: &str = "flatten";

pub fn list() -> Vec<BenchmarkInfo> {
    BenchmarkInfo::from_data_items(CATEGORY)
}

pub fn run(name: &str, runner: &BenchRunner, level: Level) -> Option<BenchmarkResult> {
    let items = get_data_items();
    let item = items.iter().find(|i| i.name == name)?;
    let expanded_strokes = item.expanded_strokes();
    let simd_variant = level_suffix(level);

    Some(runner.run(
        &format!("{CATEGORY}/{name}"),
        CATEGORY,
        name,
        simd_variant,
        #[inline(always)]
        || {
            let mut line_buf: Vec<Line> = vec![];
            let mut temp_buf: Vec<Line> = vec![];
            let mut flatten_ctx = FlattenCtx::default();

            for path in &item.fills {
                flatten::fill(level, &path.path, path.transform, &mut temp_buf, &mut flatten_ctx);
                line_buf.extend(&temp_buf);
            }

            for stroke in &expanded_strokes {
                flatten::fill(level, stroke, Affine::IDENTITY, &mut temp_buf, &mut flatten_ctx);
                line_buf.extend(&temp_buf);
            }

            std::hint::black_box(&line_buf);
        },
    ))
}
