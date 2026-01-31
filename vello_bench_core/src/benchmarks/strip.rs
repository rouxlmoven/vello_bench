// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::benchmarks::run_bench;
use crate::data::get_data_items;
use vello_common::fearless_simd::Level;
use vello_common::peniko::Fill;

pub fn register() {
    // Registration would go here for the registry-based approach
}

pub fn run_benchmarks() {
    for item in get_data_items() {
        let lines = item.lines();
        let tiles = item.sorted_tiles();

        let simd_level = Level::new();
        if !matches!(simd_level, Level::Fallback(_)) {
            let name = format!("render_strips/{}_simd", item.name);

            run_bench(&name, || {
                let mut strip_buf = vec![];
                let mut alpha_buf = vec![];

                strip_buf.clear();
                alpha_buf.clear();

                vello_common::strip::render(
                    simd_level,
                    &tiles,
                    &mut strip_buf,
                    &mut alpha_buf,
                    Fill::NonZero,
                    None,
                    &lines,
                );
                std::hint::black_box((&strip_buf, &alpha_buf));
            });
        }
    }
}
