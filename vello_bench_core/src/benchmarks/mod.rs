// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Individual benchmark modules.
//!
//! Each module exposes:
//! - `list() -> Vec<BenchmarkInfo>` — the benchmarks it provides.
//! - `run(id, runner, level) -> Option<BenchmarkResult>` — run a benchmark by name.

pub mod fine_fill;
pub mod fine_gradient;
pub mod fine_image;
pub mod fine_pack;
pub mod fine_strip;
pub mod flatten;
pub mod render_strips;
pub mod strokes;
pub mod tile;
