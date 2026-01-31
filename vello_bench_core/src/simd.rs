// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! SIMD level abstraction for benchmarking.

use serde::{Deserialize, Serialize};
use vello_common::fearless_simd::Level;

/// User-selectable SIMD level for benchmarks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimdLevel {
    /// Scalar (no SIMD).
    Scalar,
    /// x86_64 SSE4.2.
    Sse42,
    /// x86_64 AVX2.
    Avx2,
    /// ARM NEON.
    Neon,
    /// WebAssembly SIMD128.
    WasmSimd128,
}

impl SimdLevel {
    /// Returns all SIMD levels available on the current platform, ordered from best to worst.
    /// The first element is the best (fastest) available level.
    pub fn available() -> Vec<SimdLevel> {
        let mut levels = vec![];

        #[cfg(target_arch = "x86_64")]
        {
            // AVX2 is better than SSE4.2
            if std::arch::is_x86_feature_detected!("avx2") {
                levels.push(SimdLevel::Avx2);
            }
            if std::arch::is_x86_feature_detected!("sse4.2") {
                levels.push(SimdLevel::Sse42);
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            // NEON is always available on aarch64
            levels.push(SimdLevel::Neon);
        }

        #[cfg(target_arch = "wasm32")]
        {
            // Check for SIMD128 support - this would be set by the build
            #[cfg(target_feature = "simd128")]
            levels.push(SimdLevel::WasmSimd128);
        }

        // Scalar is always available as fallback (last/worst option)
        levels.push(SimdLevel::Scalar);

        levels
    }

    /// Returns the best (fastest) SIMD level available on this platform.
    pub fn best() -> SimdLevel {
        Self::available().into_iter().next().unwrap_or(SimdLevel::Scalar)
    }

    /// Returns the display name for this SIMD level.
    pub fn display_name(&self) -> &'static str {
        match self {
            SimdLevel::Scalar => "Scalar",
            SimdLevel::Sse42 => "SSE4.2",
            SimdLevel::Avx2 => "AVX2",
            SimdLevel::Neon => "NEON",
            SimdLevel::WasmSimd128 => "WASM SIMD128",
        }
    }

    /// Returns the suffix used in benchmark names for this SIMD level.
    pub fn suffix(&self) -> &'static str {
        match self {
            SimdLevel::Scalar => "scalar",
            SimdLevel::Sse42 => "sse42",
            SimdLevel::Avx2 => "avx2",
            SimdLevel::Neon => "neon",
            SimdLevel::WasmSimd128 => "wasm_simd128",
        }
    }

    /// Converts to fearless_simd Level, if available.
    /// Returns None if the requested level is not available on this platform.
    pub fn to_level(&self) -> Option<Level> {
        match self {
            SimdLevel::Scalar => Some(Level::fallback()),
            #[cfg(target_arch = "x86_64")]
            SimdLevel::Sse42 => {
                let level = Level::new();
                if level.as_sse42().is_some() {
                    Some(level)
                } else {
                    None
                }
            }
            #[cfg(target_arch = "x86_64")]
            SimdLevel::Avx2 => {
                let level = Level::new();
                if level.as_avx2().is_some() {
                    Some(level)
                } else {
                    None
                }
            }
            #[cfg(target_arch = "aarch64")]
            SimdLevel::Neon => Some(Level::new()),
            #[cfg(target_arch = "wasm32")]
            SimdLevel::WasmSimd128 => {
                #[cfg(target_feature = "simd128")]
                {
                    Some(Level::new())
                }
                #[cfg(not(target_feature = "simd128"))]
                {
                    None
                }
            }
            #[allow(unreachable_patterns)]
            _ => None,
        }
    }
}

impl std::fmt::Display for SimdLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
