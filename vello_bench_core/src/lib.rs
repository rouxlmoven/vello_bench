// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Core library for vello benchmarking.

#![allow(missing_docs, reason = "Not needed for benchmarks")]

pub mod data;
pub mod dispatch;
pub mod result;
pub mod runner;
pub mod simd;

pub use dispatch::{get_benchmark_list, run_benchmark_by_id, BenchmarkInfo};
pub use result::{BenchmarkResult, PlatformInfo, Statistics};
pub use runner::BenchRunner;
pub use simd::SimdLevel;
