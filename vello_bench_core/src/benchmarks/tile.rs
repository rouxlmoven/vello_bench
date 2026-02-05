// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::data::get_data_items;
use crate::registry::BenchmarkInfo;
use crate::result::BenchmarkResult;
use crate::runner::BenchRunner;
use crate::simd::level_suffix;
use fearless_simd::Level;
use vello_common::tile::Tiles;

const CATEGORY: &str = "tile";

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
    let simd_variant = level_suffix(level);

    Some(runner.run(
        &format!("{CATEGORY}/{name}"),
        CATEGORY,
        name,
        simd_variant,
        || {
            let mut tiler = Tiles::new(level);
            tiler.make_tiles_analytic_aa(&lines, item.width, item.height);
            std::hint::black_box(&tiler);
        },
    ))
}
