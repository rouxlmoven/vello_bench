pub mod benchmarks;
pub mod data;
pub mod registry;
pub mod result;
pub mod runner;
pub mod simd;

/// Duration of the calibration phase in milliseconds.
/// Keep in sync with `CALIBRATION_MS` in `ui/app.js`.
pub const CALIBRATION_MS: u64 = 2_000;
/// Duration of the measurement phase in milliseconds.
pub const MEASUREMENT_MS: u64 = 4_000;

pub use registry::{get_benchmark_list, run_benchmark_by_id, BenchmarkInfo};
pub use result::{BenchmarkResult, Statistics};
pub use runner::BenchRunner;
pub use simd::{SimdLevelInfo, available_level_infos, available_levels, level_from_suffix, level_suffix};
