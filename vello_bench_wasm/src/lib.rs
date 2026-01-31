// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! WASM bindings for vello benchmarks.

#![allow(missing_docs, reason = "Not needed for benchmarks")]

use wasm_bindgen::prelude::*;
use vello_bench_core::{BenchRunner, PlatformInfo, SimdLevel};

/// Initialize the WASM module.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// List all available benchmarks.
#[wasm_bindgen]
pub fn list_benchmarks() -> JsValue {
    let benchmarks = vello_bench_core::get_benchmark_list();
    serde_wasm_bindgen::to_value(&benchmarks).unwrap()
}

/// Get available SIMD levels for this platform.
#[wasm_bindgen]
pub fn get_simd_levels() -> JsValue {
    let levels = SimdLevel::available();
    let level_info: Vec<_> = levels
        .into_iter()
        .map(|l| serde_json::json!({
            "id": l.suffix(),
            "name": l.display_name(),
        }))
        .collect();
    serde_wasm_bindgen::to_value(&level_info).unwrap()
}

/// Check if SIMD128 is available.
#[wasm_bindgen]
pub fn has_simd128() -> bool {
    #[cfg(target_feature = "simd128")]
    { true }
    #[cfg(not(target_feature = "simd128"))]
    { false }
}

/// Run a single benchmark by ID.
#[wasm_bindgen]
pub fn run_benchmark(id: &str, warmup_ms: u64, measurement_ms: u64) -> JsValue {
    let runner = BenchRunner::new(warmup_ms, measurement_ms);

    match vello_bench_core::run_benchmark_by_id(&runner, id) {
        Some(result) => serde_wasm_bindgen::to_value(&result).unwrap(),
        None => JsValue::NULL,
    }
}

/// Get platform information.
#[wasm_bindgen]
pub fn get_platform_info() -> JsValue {
    let info = PlatformInfo::detect();
    serde_wasm_bindgen::to_value(&info).unwrap()
}
