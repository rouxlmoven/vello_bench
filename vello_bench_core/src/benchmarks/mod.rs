// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Individual benchmark modules.
//!
//! Each module exposes:
//! - `list() -> Vec<BenchmarkInfo>` — the benchmarks it provides.
//! - `run(name, runner, level) -> Option<BenchmarkResult>` — run a benchmark by name.

pub mod fine;
pub mod flatten;
pub mod render_strips;
pub mod strokes;
pub mod tile;
