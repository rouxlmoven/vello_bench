use serde::{Deserialize, Serialize};
use vello_common::fearless_simd::Level;

/// Information about a SIMD level, suitable for serialization to frontends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimdLevelInfo {
    pub id: String,
    pub name: String,
}

/// Returns all SIMD levels available on the current platform, ordered from best to worst.
///
/// Always includes the best available level (via `Level::new()`) and the scalar fallback.
/// On platforms where the best level is already scalar, only one entry is returned.
pub fn available_levels() -> Vec<Level> {
    let best = Level::new();
    if matches!(best, Level::Fallback(_)) {
        vec![best]
    } else {
        vec![best, Level::fallback()]
    }
}

/// Returns a short suffix string identifying the SIMD level (e.g., "avx2", "neon", "scalar").
pub fn level_suffix(level: Level) -> &'static str {
    match level {
        Level::Fallback(_) => "scalar",
        #[cfg(target_arch = "aarch64")]
        Level::Neon(_) => "neon",
        #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
        Level::WasmSimd128(_) => "wasm_simd128",
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        Level::Sse4_2(_) => "sse42",
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        Level::Avx2(_) => "avx2",
        _ => "unknown",
    }
}

/// Returns a human-readable display name for the SIMD level (e.g., "AVX2", "NEON", "Scalar").
pub fn level_display_name(level: Level) -> &'static str {
    match level {
        Level::Fallback(_) => "Scalar",
        #[cfg(target_arch = "aarch64")]
        Level::Neon(_) => "NEON",
        #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
        Level::WasmSimd128(_) => "WASM SIMD128",
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        Level::Sse4_2(_) => "SSE4.2",
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        Level::Avx2(_) => "AVX2",
        _ => "Unknown",
    }
}

/// Parse a SIMD level from a suffix string (as returned by `level_suffix`).
/// Falls back to `Level::new()` (best available) if the string is unrecognized.
///
/// Note: fearless_simd's `Level::new()` always returns the best available level.
/// Requesting a specific sub-level (e.g. "sse42" on an AVX2 machine) is not
/// currently supported by fearless_simd, so we return the best available level
/// in those cases.
pub fn level_from_suffix(s: &str) -> Level {
    match s {
        "scalar" => Level::fallback(),
        _ => Level::new(),
    }
}

/// Get `SimdLevelInfo` structs for all available levels, suitable for sending to a frontend.
pub fn available_level_infos() -> Vec<SimdLevelInfo> {
    available_levels()
        .into_iter()
        .map(|l| SimdLevelInfo {
            id: level_suffix(l).to_string(),
            name: level_display_name(l).to_string(),
        })
        .collect()
}
