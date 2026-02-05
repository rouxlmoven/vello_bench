pub mod benchmarks;
pub mod data;
pub mod registry;
pub mod result;
pub mod runner;
pub mod simd;

pub use registry::{get_benchmark_list, run_benchmark_by_id, BenchmarkInfo};
pub use result::{BenchmarkResult, Statistics};
pub use runner::BenchRunner;
pub use simd::{SimdLevelInfo, available_level_infos, available_levels, level_from_suffix, level_suffix};
