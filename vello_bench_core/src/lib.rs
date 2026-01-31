// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Core library for vello benchmarking.
//!
//! This crate provides shared functionality for the vello benchmark suite,
//! including benchmark registration, execution, and result handling.

#![allow(missing_docs, reason = "Not needed for benchmarks")]
#![allow(dead_code, reason = "Might be unused on platforms not supporting SIMD")]

pub mod benchmarks;
pub mod data;
pub mod registry;
pub mod result;
pub mod runner;
pub mod simd;

// Re-export commonly used items
pub use registry::{list_benchmarks, register, run_benchmark, BenchmarkMetadata, REGISTRY};
pub use result::{BenchmarkResult, PlatformInfo, Statistics};
pub use runner::BenchRunner;
pub use simd::SimdLevel;

// Re-export benchmark runner function for CLI compatibility
pub use benchmarks::run_all_benchmarks;

// Re-export run_bench for the vello_bench macro (it uses crate::run_bench)
pub use benchmarks::run_bench;
