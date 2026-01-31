// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::benchmarks::run_bench;
use crate::data::get_data_items;
use vello_common::tile::Tiles;
use vello_cpu::Level;

pub fn register() {
    // Registration would go here for the registry-based approach
    // For now, benchmarks are run directly
}

pub fn run_benchmarks() {
    for item in get_data_items() {
        let lines = item.lines();
        let name = format!("tile/{}", item.name);

        run_bench(&name, || {
            let mut tiler = Tiles::new(Level::new());
            tiler.make_tiles_analytic_aa(&lines, item.width, item.height);
            std::hint::black_box(&tiler);
        });
    }
}
