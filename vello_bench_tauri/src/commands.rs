// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Tauri commands for benchmark operations.

use std::sync::LazyLock;
use tokio::sync::Mutex;
use vello_bench_core::{BenchRunner, BenchmarkInfo, BenchmarkResult, PlatformInfo, SimdLevel};

/// Mutex to ensure only one benchmark runs at a time.
static BENCHMARK_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

/// SIMD level info for the frontend.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SimdLevelInfo {
    pub id: String,
    pub name: String,
}

/// Get list of available benchmarks.
#[tauri::command]
pub fn list_benchmarks() -> Vec<BenchmarkInfo> {
    vello_bench_core::get_benchmark_list()
}

/// Get available SIMD levels.
#[tauri::command]
pub fn get_simd_levels() -> Vec<SimdLevelInfo> {
    SimdLevel::available()
        .into_iter()
        .map(|l| SimdLevelInfo {
            id: l.suffix().to_string(),
            name: l.display_name().to_string(),
        })
        .collect()
}

/// Get platform info.
#[tauri::command]
pub fn get_platform_info() -> PlatformInfo {
    PlatformInfo::detect()
}

/// Run a single benchmark (async, runs in background thread).
#[tauri::command]
pub async fn run_benchmark(
    id: String,
    simd_level: String,
    warmup_ms: u64,
    measurement_ms: u64,
) -> Option<BenchmarkResult> {
    // Acquire lock to ensure only one benchmark runs at a time
    let _guard = BENCHMARK_LOCK.lock().await;

    // Run the benchmark in a blocking thread to not block the async runtime
    tokio::task::spawn_blocking(move || {
        let runner = BenchRunner::new(warmup_ms, measurement_ms);
        vello_bench_core::run_benchmark_by_id(&runner, &id)
    })
    .await
    .ok()
    .flatten()
}
