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

/// Platform information for a benchmark run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    /// Architecture (e.g., "x86_64", "aarch64", "wasm32").
    pub arch: String,
    /// Operating system (e.g., "macos", "windows", "linux", "browser").
    pub os: String,
    /// Available SIMD features.
    pub simd_features: Vec<String>,
}

impl PlatformInfo {
    /// Detect current platform information.
    pub fn detect() -> Self {
        let arch = std::env::consts::ARCH.to_string();

        #[cfg(target_arch = "wasm32")]
        let os = "browser".to_string();
        #[cfg(not(target_arch = "wasm32"))]
        let os = std::env::consts::OS.to_string();

        let mut simd_features = vec![];

        #[cfg(target_arch = "x86_64")]
        {
            if std::arch::is_x86_feature_detected!("avx2") {
                simd_features.push("avx2".to_string());
            }
            if std::arch::is_x86_feature_detected!("sse4.2") {
                simd_features.push("sse4.2".to_string());
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            // NEON is always available on aarch64
            simd_features.push("neon".to_string());
        }

        #[cfg(target_arch = "wasm32")]
        {
            // WASM SIMD detection would need to be done at runtime via JavaScript
            simd_features.push("scalar".to_string());
        }

        if simd_features.is_empty() {
            simd_features.push("scalar".to_string());
        }

        Self {
            arch,
            os,
            simd_features,
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
    /// Platform information.
    pub platform: PlatformInfo,
}
