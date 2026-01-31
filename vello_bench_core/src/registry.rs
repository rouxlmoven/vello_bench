// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Benchmark registry for discovering and running benchmarks.

use crate::result::BenchmarkResult;
use crate::runner::BenchRunner;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

/// Metadata about a benchmark.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetadata {
    /// Full benchmark ID (e.g., "fine/fill/opaque_short_u8_neon").
    pub id: String,
    /// Category (e.g., "fine/fill").
    pub category: String,
    /// Benchmark name without SIMD suffix (e.g., "opaque_short").
    pub name: String,
    /// SIMD variant (e.g., "u8_neon", "scalar").
    pub simd_variant: String,
}

impl BenchmarkMetadata {
    /// Parse benchmark metadata from a full benchmark path.
    /// Expected format: "category/subcategory/name_simd_variant"
    pub fn from_path(path: &str) -> Self {
        let parts: Vec<&str> = path.rsplitn(2, '/').collect();
        let (name_with_variant, category) = if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            (path, "")
        };

        // Parse name and SIMD variant (e.g., "opaque_short_u8_neon" -> "opaque_short", "u8_neon")
        let (name, simd_variant) = parse_name_variant(name_with_variant);

        Self {
            id: path.to_string(),
            category: category.to_string(),
            name: name.to_string(),
            simd_variant: simd_variant.to_string(),
        }
    }
}

/// Parse a benchmark name into base name and SIMD variant.
fn parse_name_variant(name_with_variant: &str) -> (&str, &str) {
    // Known SIMD suffixes
    let suffixes = [
        "_u8_neon",
        "_u8_avx2",
        "_u8_sse42",
        "_u8_wasm",
        "_u8_scalar",
        "_f32_neon",
        "_f32_avx2",
        "_f32_sse42",
        "_f32_wasm",
        "_f32_scalar",
        "_simd",
        "_scalar",
    ];

    for suffix in suffixes {
        if let Some(base) = name_with_variant.strip_suffix(suffix) {
            return (base, &suffix[1..]); // Remove leading underscore
        }
    }

    (name_with_variant, "default")
}

/// A registered benchmark function.
pub type BenchmarkFn = Box<dyn Fn(&BenchRunner) -> BenchmarkResult + Send + Sync>;

/// Registry of all benchmarks.
pub struct BenchmarkRegistry {
    benchmarks: HashMap<String, (BenchmarkMetadata, BenchmarkFn)>,
    /// Ordered list of benchmark IDs for consistent iteration.
    order: Vec<String>,
}

impl BenchmarkRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            benchmarks: HashMap::new(),
            order: Vec::new(),
        }
    }

    /// Register a benchmark.
    pub fn register<F>(&mut self, id: &str, func: F)
    where
        F: Fn(&BenchRunner) -> BenchmarkResult + Send + Sync + 'static,
    {
        let metadata = BenchmarkMetadata::from_path(id);
        self.order.push(id.to_string());
        self.benchmarks
            .insert(id.to_string(), (metadata, Box::new(func)));
    }

    /// List all registered benchmarks.
    pub fn list(&self) -> Vec<&BenchmarkMetadata> {
        self.order
            .iter()
            .filter_map(|id| self.benchmarks.get(id).map(|(m, _)| m))
            .collect()
    }

    /// List benchmarks by category.
    pub fn list_by_category(&self, category: &str) -> Vec<&BenchmarkMetadata> {
        self.list()
            .into_iter()
            .filter(|m| m.category == category || m.category.starts_with(&format!("{}/", category)))
            .collect()
    }

    /// Get all unique categories.
    pub fn categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self
            .list()
            .iter()
            .map(|m| m.category.clone())
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }

    /// Run a single benchmark by ID.
    pub fn run(&self, id: &str, runner: &BenchRunner) -> Option<BenchmarkResult> {
        self.benchmarks.get(id).map(|(_, func)| func(runner))
    }

    /// Run all benchmarks.
    pub fn run_all(&self, runner: &BenchRunner) -> Vec<BenchmarkResult> {
        self.order
            .iter()
            .filter_map(|id| self.run(id, runner))
            .collect()
    }

    /// Run benchmarks matching a category prefix.
    pub fn run_category(&self, category: &str, runner: &BenchRunner) -> Vec<BenchmarkResult> {
        self.order
            .iter()
            .filter(|id| {
                self.benchmarks
                    .get(*id)
                    .map(|(m, _)| {
                        m.category == category || m.category.starts_with(&format!("{}/", category))
                    })
                    .unwrap_or(false)
            })
            .filter_map(|id| self.run(id, runner))
            .collect()
    }
}

impl Default for BenchmarkRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global benchmark registry.
pub static REGISTRY: LazyLock<RwLock<BenchmarkRegistry>> =
    LazyLock::new(|| RwLock::new(BenchmarkRegistry::new()));

/// Register a benchmark in the global registry.
pub fn register<F>(id: &str, func: F)
where
    F: Fn(&BenchRunner) -> BenchmarkResult + Send + Sync + 'static,
{
    REGISTRY.write().unwrap().register(id, func);
}

/// List all benchmarks in the global registry.
pub fn list_benchmarks() -> Vec<BenchmarkMetadata> {
    REGISTRY.read().unwrap().list().into_iter().cloned().collect()
}

/// Run a benchmark from the global registry.
pub fn run_benchmark(id: &str, runner: &BenchRunner) -> Option<BenchmarkResult> {
    REGISTRY.read().unwrap().run(id, runner)
}
