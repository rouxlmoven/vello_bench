// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::data::get_data_items;
use crate::registry::BenchmarkInfo;
use crate::result::BenchmarkResult;
use crate::runner::BenchRunner;
use crate::simd::level_suffix;
use fearless_simd::Level;
use vello_common::peniko::Fill;
use vello_common::strip::Strip;

const CATEGORY: &str = "render_strips";

pub fn list() -> Vec<BenchmarkInfo> {
    get_data_items()
        .iter()
        .map(|item| BenchmarkInfo {
            id: format!("{CATEGORY}/{}", item.name),
            category: CATEGORY.into(),
            name: item.name.clone(),
        })
        .collect()
}

pub fn run(name: &str, runner: &BenchRunner, level: Level) -> Option<BenchmarkResult> {
    let items = get_data_items();
    let item = items.iter().find(|i| i.name == name)?;
    let lines = item.lines();
    let tiles = item.sorted_tiles();
    let simd_variant = level_suffix(level);

    Some(runner.run(
        &format!("{CATEGORY}/{name}"),
        CATEGORY,
        name,
        simd_variant,
        || {
            let mut strip_buf: Vec<Strip> = vec![];
            let mut alpha_buf: Vec<u8> = vec![];

            vello_common::strip::render(
                level,
                &tiles,
                &mut strip_buf,
                &mut alpha_buf,
                Fill::NonZero,
                None,
                &lines,
            );

            std::hint::black_box(&strip_buf);
        },
    ))
}
