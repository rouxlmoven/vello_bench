pub mod benchmarks;
pub mod data;
pub mod registry;
pub mod result;
pub mod runner;
pub mod scenes;
pub mod screenshot;
pub mod simd;

pub use fearless_simd::Level;
pub use registry::{BenchmarkInfo, get_benchmark_list, run_benchmark_by_id};
pub use result::{BenchmarkResult, Statistics};
pub use runner::BenchRunner;
pub use simd::{
    SimdLevelInfo, available_level_infos, available_levels, level_from_suffix, level_suffix,
};
