// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Benchmark implementations.

pub mod fine;
pub mod flatten;
pub mod strip;
pub mod tile;

use crate::runner::BenchRunner;

/// Seed for random number generation in benchmarks.
pub const SEED: [u8; 32] = [0; 32];

/// Initialize all benchmarks by running their registration functions.
pub fn register_all() {
    tile::register();
    flatten::register();
    strip::register();
    fine::register();
}

/// Run all benchmarks (for CLI compatibility).
pub fn run_all_benchmarks() {
    let runner = BenchRunner::default_timing();

    println!("Vello Benchmark Suite");
    println!("Measurement: {}ms", runner.measurement_ms);

    section("Tile");
    tile::run_benchmarks();

    section("Flatten");
    flatten::run_benchmarks();

    section("Strip Rendering");
    strip::run_benchmarks();

    section("Fine - Fill");
    fine::fill::run_benchmarks();

    section("Fine - Strip");
    fine::strip::run_benchmarks();

    section("Fine - Pack");
    fine::pack::run_benchmarks();

    section("Fine - Gradient");
    fine::gradient::run_benchmarks();

    section("Fine - Image");
    fine::image::run_benchmarks();

    println!("\n{}", "=".repeat(70));
    println!("Benchmarks complete.");
}

/// Print a section header.
pub fn section(name: &str) {
    println!("\n{}", "=".repeat(70));
    println!("{}", name);
    println!("{}", "=".repeat(70));
}

/// Format time with appropriate unit for display.
fn format_time(mean_ns: f64) -> (f64, &'static str) {
    if mean_ns >= 1_000_000_000.0 {
        (mean_ns / 1_000_000_000.0, "s ")
    } else if mean_ns >= 1_000_000.0 {
        (mean_ns / 1_000_000.0, "ms")
    } else if mean_ns >= 1_000.0 {
        (mean_ns / 1_000.0, "µs")
    } else {
        (mean_ns, "ns")
    }
}

/// Run a named benchmark and print results.
pub fn run_bench<F>(name: &str, mut f: F)
where
    F: FnMut(),
{
    let runner = BenchRunner::default_timing();
    run_bench_with_runner(name, &runner, &mut f);
}

/// Run a named benchmark with a custom runner and print results.
pub fn run_bench_with_runner<F>(name: &str, runner: &BenchRunner, f: &mut F)
where
    F: FnMut(),
{
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::Instant;

        let target_calibration_ns = 500_000_000.0; // 500ms
        let target_measurement_ns = runner.measurement_ms as f64 * 1_000_000.0;

        // Calibration phase: find batch size that takes ~500ms
        let mut batch_size = 1usize;
        let mut batch_time_ns;

        loop {
            let start = Instant::now();
            for _ in 0..batch_size {
                f();
            }
            batch_time_ns = start.elapsed().as_nanos() as f64;

            if batch_time_ns >= target_calibration_ns {
                break;
            }

            batch_size *= 2;
        }

        // Calculate iterations needed for target measurement time
        let iters_per_ns = batch_size as f64 / batch_time_ns;
        let total_iters = (iters_per_ns * target_measurement_ns).ceil() as usize;

        // Single measurement
        let start = Instant::now();
        for _ in 0..total_iters {
            f();
        }
        let elapsed_ns = start.elapsed().as_nanos() as f64;
        let mean_ns = elapsed_ns / total_iters as f64;

        let (mean_scaled, unit) = format_time(mean_ns);

        println!(
            "{:50} {:>10.3} {} ({} iters)",
            name, mean_scaled, unit, total_iters
        );
    }

    #[cfg(target_arch = "wasm32")]
    {
        // On WASM, we'd use web-sys Performance API
        let _ = (name, runner, f);
    }
}
