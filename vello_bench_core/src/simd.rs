use serde::{Deserialize, Serialize};
use fearless_simd::Level;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use fearless_simd::{Avx2, Sse4_2};
#[cfg(target_arch = "aarch64")]
use fearless_simd::Neon;
use fearless_simd::Fallback;

/// Information about a SIMD level, suitable for serialization to frontends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimdLevelInfo {
    pub id: String,
    pub name: String,
}

/// Returns all SIMD levels available on the current platform, ordered from best to worst.
#[allow(unsafe_code)]
pub fn available_levels() -> Vec<Level> {
    let mut levels = Vec::new();

    #[cfg(target_arch = "aarch64")]
    if std::arch::is_aarch64_feature_detected!("neon") {
        levels.push(Level::Neon(unsafe { Neon::new_unchecked() }));
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if std::arch::is_x86_feature_detected!("avx2")
            && std::arch::is_x86_feature_detected!("fma")
        {
            levels.push(Level::Avx2(unsafe { Avx2::new_unchecked() }));
        }
        if std::arch::is_x86_feature_detected!("sse4.2") {
            levels.push(Level::Sse4_2(unsafe { Sse4_2::new_unchecked() }));
        }
    }

    #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
    {
        levels.push(Level::WasmSimd128(fearless_simd::WasmSimd128::new_unchecked()));
    }

    levels.push(Level::Fallback(Fallback::new()));
    levels
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
#[allow(unsafe_code)]
pub fn level_from_suffix(s: &str) -> Level {
    match s {
        "scalar" => Level::Fallback(Fallback::new()),
        #[cfg(target_arch = "aarch64")]
        "neon" => Level::Neon(unsafe { Neon::new_unchecked() }),
        #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
        "wasm_simd128" => Level::WasmSimd128(fearless_simd::WasmSimd128::new_unchecked()),
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        "sse42" => Level::Sse4_2(unsafe { Sse4_2::new_unchecked() }),
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        "avx2" => Level::Avx2(unsafe { Avx2::new_unchecked() }),
        _ => panic!("unknown SIMD level suffix: {s:?}"),
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
