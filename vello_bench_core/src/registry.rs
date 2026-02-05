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

impl BenchmarkInfo {
    /// Build a list from static benchmark names.
    pub fn from_names(category: &str, names: &[&str]) -> Vec<Self> {
        names
            .iter()
            .map(|name| Self {
                id: format!("{category}/{name}"),
                category: category.into(),
                name: (*name).into(),
            })
            .collect()
    }

    /// Build a list from data items (one benchmark per SVG).
    pub fn from_data_items(category: &str) -> Vec<Self> {
        crate::data::get_data_items()
            .iter()
            .map(|item| Self {
                id: format!("{category}/{}", item.name),
                category: category.into(),
                name: item.name.clone(),
            })
            .collect()
    }
}

/// Get the complete list of all available benchmarks.
pub fn get_benchmark_list() -> Vec<BenchmarkInfo> {
    let mut benchmarks = Vec::new();

    benchmarks.extend(fine::fill::list());
    benchmarks.extend(fine::gradient::list());
    benchmarks.extend(fine::image::list());
    benchmarks.extend(fine::pack::list());
    benchmarks.extend(fine::strip::list());
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
        return fine::fill::run(name, runner, level);
    }
    if let Some(name) = id.strip_prefix("fine/gradient/") {
        return fine::gradient::run(name, runner, level);
    }
    if let Some(name) = id.strip_prefix("fine/image/") {
        return fine::image::run(name, runner, level);
    }
    if let Some(name) = id.strip_prefix("fine/pack/") {
        return fine::pack::run(name, runner, level);
    }
    if let Some(name) = id.strip_prefix("fine/strip/") {
        return fine::strip::run(name, runner, level);
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
