//! Tauri commands for benchmark operations.

use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;
use tokio::sync::Mutex;
use vello_bench_core::{
    BenchRunner, BenchmarkInfo, BenchmarkResult, PlatformInfo, SimdLevelInfo,
    available_level_infos, level_from_suffix,
};

/// Mutex to ensure only one benchmark runs at a time.
static BENCHMARK_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

/// Get list of available benchmarks.
#[tauri::command]
pub fn list_benchmarks() -> Vec<BenchmarkInfo> {
    vello_bench_core::get_benchmark_list()
}

/// Get available SIMD levels.
#[tauri::command]
pub fn get_simd_levels() -> Vec<SimdLevelInfo> {
    available_level_infos()
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
        let level = level_from_suffix(&simd_level);
        let runner = BenchRunner::new(warmup_ms, measurement_ms);
        vello_bench_core::run_benchmark_by_id(&runner, &id, level)
    })
    .await
    .ok()
    .flatten()
}

/// Get the directory for storing reference files.
fn get_references_dir() -> PathBuf {
    // Use the user's home directory with a .vello-bench subfolder
    let home = dirs::data_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join("vello-bench").join("references")
}

/// Metadata about a saved reference file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReferenceInfo {
    pub name: String,
    pub created_at: u64,
    pub benchmark_count: usize,
    pub platform: Option<PlatformInfo>,
}

/// Save benchmark results as a named reference.
#[tauri::command]
pub fn save_reference(name: String, results: Vec<BenchmarkResult>) -> Result<(), String> {
    let dir = get_references_dir();

    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create references directory: {e}"))?;

    // Sanitize the name for use as a filename
    let safe_name: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    if safe_name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    let file_path = dir.join(format!("{safe_name}.json"));

    let json = serde_json::to_string_pretty(&results)
        .map_err(|e| format!("Failed to serialize results: {e}"))?;

    fs::write(&file_path, json).map_err(|e| format!("Failed to write reference file: {e}"))?;

    Ok(())
}

/// List all saved reference files.
#[tauri::command]
pub fn list_references() -> Vec<ReferenceInfo> {
    let dir = get_references_dir();

    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };

    let mut references = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            // Try to get file metadata
            let created_at = fs::metadata(&path)
                .and_then(|m| m.modified())
                .map(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0)
                })
                .unwrap_or(0);

            // Try to read and parse to get benchmark count and platform
            let (benchmark_count, platform) = fs::read_to_string(&path)
                .ok()
                .and_then(|content| serde_json::from_str::<Vec<BenchmarkResult>>(&content).ok())
                .map(|results| {
                    let count = results.len();
                    let platform = results.first().map(|r| r.platform.clone());
                    (count, platform)
                })
                .unwrap_or((0, None));

            references.push(ReferenceInfo {
                name,
                created_at,
                benchmark_count,
                platform,
            });
        }
    }

    // Sort by creation time, newest first
    references.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    references
}

/// Load a reference file by name.
#[tauri::command]
pub fn load_reference(name: String) -> Result<Vec<BenchmarkResult>, String> {
    let dir = get_references_dir();
    let file_path = dir.join(format!("{name}.json"));

    let content =
        fs::read_to_string(&file_path).map_err(|e| format!("Failed to read reference file: {e}"))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse reference file: {e}"))
}

/// Delete a reference file by name.
#[tauri::command]
pub fn delete_reference(name: String) -> Result<(), String> {
    let dir = get_references_dir();
    let file_path = dir.join(format!("{name}.json"));

    fs::remove_file(&file_path).map_err(|e| format!("Failed to delete reference file: {e}"))
}
