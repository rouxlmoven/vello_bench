// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Benchmark registry — chains individual benchmark modules together.

use crate::benchmarks::*;
use crate::result::BenchmarkResult;
use crate::runner::BenchRunner;
use fearless_simd::Level;
use serde::{Deserialize, Serialize};

/// Benchmark info for the frontend/CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkInfo {
    pub id: String,
    pub category: String,
    pub name: String,
}

/// Get the complete list of all available benchmarks.
pub fn get_benchmark_list() -> Vec<BenchmarkInfo> {
    let mut benchmarks = Vec::new();

    benchmarks.extend(fine_fill::list());
    benchmarks.extend(fine_gradient::list());
    benchmarks.extend(fine_image::list());
    benchmarks.extend(fine_pack::list());
    benchmarks.extend(fine_strip::list());
    benchmarks.extend(tile::list());
    benchmarks.extend(flatten::list());
    benchmarks.extend(strokes::list());
    benchmarks.extend(render_strips::list());

    benchmarks
}

/// Run a benchmark by ID with a specific SIMD level.
/// Returns None if the benchmark ID is not found.
pub fn run_benchmark_by_id(
    runner: &BenchRunner,
    id: &str,
    level: Level,
) -> Option<BenchmarkResult> {
    // Try each category by stripping its prefix and delegating to the module.
    if let Some(name) = id.strip_prefix("fine/fill/") {
        return fine_fill::run(name, runner, level);
    }
    if let Some(name) = id.strip_prefix("fine/gradient/") {
        return fine_gradient::run(name, runner, level);
    }
    if let Some(name) = id.strip_prefix("fine/image/") {
        return fine_image::run(name, runner, level);
    }
    if let Some(name) = id.strip_prefix("fine/pack/") {
        return fine_pack::run(name, runner, level);
    }
    if let Some(name) = id.strip_prefix("fine/strip/") {
        return fine_strip::run(name, runner, level);
    }
    if let Some(name) = id.strip_prefix("tile/") {
        return tile::run(name, runner, level);
    }
    if let Some(name) = id.strip_prefix("flatten/") {
        return flatten::run(name, runner, level);
    }
    if let Some(name) = id.strip_prefix("strokes/") {
        return strokes::run(name, runner, level);
    }
    if let Some(name) = id.strip_prefix("render_strips/") {
        return render_strips::run(name, runner, level);
    }

    None
}
