// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(missing_docs, reason = "Not needed for benchmarks")]
#![allow(dead_code, reason = "Might be unused on platforms not supporting SIMD")]

use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

mod data;
mod fine;
mod flatten;
mod glyph;
mod integration;
mod strip;
mod tile;

pub(crate) const SEED: [u8; 32] = [0; 32];
pub static DATA_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data"));

const WARMUP_DURATION: Duration = Duration::from_secs(1);
const BENCH_DURATION: Duration = Duration::from_secs(3);

/// A simple benchmarking harness.
pub struct Bencher {
    name: String,
    samples: Vec<Duration>,
}

impl Bencher {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            samples: Vec::new(),
        }
    }

    /// Run a benchmark function with warmup and measurement phases.
    pub fn bench<F>(&mut self, mut f: F)
    where
        F: FnMut(),
    {
        // Warmup phase
        let warmup_start = Instant::now();
        while warmup_start.elapsed() < WARMUP_DURATION {
            f();
        }

        // Measurement phase - collect individual samples
        self.samples.clear();
        let bench_start = Instant::now();
        while bench_start.elapsed() < BENCH_DURATION {
            let iter_start = Instant::now();
            f();
            self.samples.push(iter_start.elapsed());
        }
    }

    /// Print the benchmark results.
    pub fn report(&self) {
        if self.samples.is_empty() {
            println!("{}: no samples collected", self.name);
            return;
        }

        let times_ns: Vec<f64> = self.samples.iter().map(|d| d.as_nanos() as f64).collect();
        let n = times_ns.len() as f64;

        let mean = times_ns.iter().sum::<f64>() / n;
        let variance = times_ns.iter().map(|t| (t - mean).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();

        let (mean_scaled, std_scaled, unit) = format_time(mean, std_dev);

        println!(
            "{:50} {:>10.3} {} ± {:>8.3} {} ({} iters)",
            self.name,
            mean_scaled,
            unit,
            std_scaled,
            unit,
            self.samples.len()
        );
    }
}

/// Format time with appropriate unit.
fn format_time(mean_ns: f64, std_ns: f64) -> (f64, f64, &'static str) {
    if mean_ns >= 1_000_000_000.0 {
        (mean_ns / 1_000_000_000.0, std_ns / 1_000_000_000.0, "s ")
    } else if mean_ns >= 1_000_000.0 {
        (mean_ns / 1_000_000.0, std_ns / 1_000_000.0, "ms")
    } else if mean_ns >= 1_000.0 {
        (mean_ns / 1_000.0, std_ns / 1_000.0, "µs")
    } else {
        (mean_ns, std_ns, "ns")
    }
}

/// Run a named benchmark.
pub fn run_bench<F>(name: &str, mut f: F)
where
    F: FnMut(),
{
    let mut bencher = Bencher::new(name);
    bencher.bench(&mut f);
    bencher.report();
}

/// Print a section header.
pub fn section(name: &str) {
    println!("\n{}", "=".repeat(70));
    println!("{}", name);
    println!("{}", "=".repeat(70));
}

fn main() {
    println!("Vello Benchmark Suite");
    println!(
        "Warmup: {:?}, Measurement: {:?}",
        WARMUP_DURATION, BENCH_DURATION
    );

    section("Tile");
    tile::run_benchmarks();

    section("Flatten");
    flatten::run_benchmarks();

    section("Strip Rendering");
    strip::run_benchmarks();

    section("Glyph");
    glyph::run_benchmarks();

    section("Fine - Fill");
    fine::fill::run_benchmarks();

    section("Fine - Strip");
    fine::strip::run_benchmarks();

    section("Fine - Pack");
    fine::pack::run_benchmarks();

    section("Fine - Gradient");
    fine::gradient::run_benchmarks();

    section("Fine - Rounded Blurred Rect");
    fine::rounded_blurred_rect::run_benchmarks();

    section("Fine - Blend");
    fine::blend::run_benchmarks();

    section("Fine - Image");
    fine::image::run_benchmarks();

    section("Integration");
    integration::run_benchmarks();

    println!("\n{}", "=".repeat(70));
    println!("Benchmarks complete.");
}
