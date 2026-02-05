use serde::{Deserialize, Serialize};

/// Statistics from a benchmark run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    /// Mean time in nanoseconds.
    pub mean_ns: f64,
    /// Number of iterations.
    pub iterations: usize,
}

impl Statistics {
    /// Create statistics from a single measurement.
    pub fn from_measurement(total_time_ns: f64, iterations: usize) -> Self {
        Self {
            mean_ns: total_time_ns / iterations as f64,
            iterations,
        }
    }
}

/// Result from running a single benchmark.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Full benchmark ID (e.g., "fine/fill/opaque_short").
    pub id: String,
    /// Category (e.g., "fine/fill").
    pub category: String,
    /// Benchmark name (e.g., "opaque_short").
    pub name: String,
    /// SIMD variant used (e.g., "u8_neon", "scalar").
    pub simd_variant: String,
    /// Benchmark statistics.
    pub statistics: Statistics,
    /// Timestamp when benchmark was run (milliseconds since epoch).
    pub timestamp_ms: u64,
}
