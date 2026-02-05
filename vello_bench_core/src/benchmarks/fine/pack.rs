// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::registry::BenchmarkInfo;
use crate::result::BenchmarkResult;
use crate::runner::BenchRunner;
use crate::simd::level_suffix;
use fearless_simd::{Level, dispatch};
use vello_common::coarse::WideTile;
use vello_common::tile::Tile;
use vello_cpu::fine::{Fine, U8Kernel, SCRATCH_BUF_SIZE};
use vello_cpu::region::Regions;

const NAMES: &[&str] = &["block", "regular"];
const CATEGORY: &str = "fine/pack";

pub fn list() -> Vec<BenchmarkInfo> {
    BenchmarkInfo::from_names(CATEGORY, NAMES)
}

pub fn run(name: &str, runner: &BenchRunner, level: Level) -> Option<BenchmarkResult> {
    if !NAMES.contains(&name) {
        return None;
    }

    let width = match name {
        "regular" => WideTile::WIDTH - 1,
        _ => WideTile::WIDTH,
    };

    let simd_variant = level_suffix(level);

    Some(dispatch!(level, simd => {
        let fine = Fine::<_, U8Kernel>::new(simd);

        runner.run(
            &format!("{CATEGORY}/{name}"),
            CATEGORY,
            name,
            simd_variant,
            #[inline(always)]
            || {
                let mut buf = vec![0; SCRATCH_BUF_SIZE];
                let mut regions = Regions::new(width, Tile::HEIGHT, &mut buf);
                regions.update_regions(|region| {
                    fine.pack(region);
                });
                std::hint::black_box(&regions);
            },
        )
    }))
}
