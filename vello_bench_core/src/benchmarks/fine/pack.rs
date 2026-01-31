// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use vello_bench_macros::vello_bench;
use vello_common::coarse::WideTile;
use vello_common::fearless_simd::Simd;
use vello_common::tile::Tile;
use vello_cpu::fine::{Fine, FineKernel, SCRATCH_BUF_SIZE};
use vello_cpu::region::Regions;

#[vello_bench]
fn pack_block<S: Simd, T: FineKernel<S>>(fine: &mut Fine<S, T>) {
    let mut buf = vec![0; SCRATCH_BUF_SIZE];
    let mut regions = Regions::new(WideTile::WIDTH, Tile::HEIGHT, &mut buf);
    regions.update_regions(|region| {
        fine.pack(region);
    });
    std::hint::black_box(&regions);
}

#[vello_bench]
fn pack_regular<S: Simd, T: FineKernel<S>>(fine: &mut Fine<S, T>) {
    let mut buf = vec![0; SCRATCH_BUF_SIZE];
    let mut regions = Regions::new(WideTile::WIDTH - 1, Tile::HEIGHT, &mut buf);
    regions.update_regions(|region| {
        fine.pack(region);
    });
    std::hint::black_box(&regions);
}

pub fn run_benchmarks() {
    pack_block();
    pack_regular();
}
