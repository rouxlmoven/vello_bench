use crate::result::{BenchmarkResult, Statistics};

/// Per-iteration performance marks are only emitted when the total iteration
/// count stays at or below this threshold. This avoids flooding the browser
/// Performance timeline (and adding measurable overhead) for very fast CPU
/// micro-benchmarks that run millions of iterations.  GPU/WebGL benchmarks
/// typically have far fewer iterations and always receive marks.
const MAX_MARKED_ITERS: usize = 10_000;

#[derive(Debug, Clone)]
pub struct BenchRunner {
    pub warmup: u64,
    pub iterations: u64,
}

impl BenchRunner {
    pub fn new(warmup: u64, iterations: u64) -> Self {
        Self { warmup, iterations }
    }
}

impl BenchRunner {
    /// Runs `self.warmup` iterations of `f.
    fn warmup<F>(&self, mut f: F)
    where
        F: FnMut(),
    {
        for _ in 0..self.warmup {
            f();
        }
    }

    /// Bulk-timing measurement: times the entire loop as a single span.
    ///
    /// No per-iteration `performance.mark()` calls are emitted — use
    /// [`Self::measure_per_iteration_with_frame_wait`] when DevTools per-iteration marks are
    /// needed (e.g. GPU benchmarks).
    fn measure<F, T: Timer>(
        timer: &T,
        mut f: F,
        total_iters: usize,
    ) -> Statistics
    where
        F: FnMut(),
    {
        let start = timer.now();
        for _ in 0..total_iters {
            f();
        }
        let elapsed_ns = timer.elapsed_ns(start);

        Statistics::from_measurement(elapsed_ns, total_iters)
    }

    /// Run the measurement phase with **per-iteration timing** and an untimed
    /// frame wait between iterations.
    ///
    /// Each call to `f()` is timed individually and the elapsed durations are
    /// accumulated. Between iterations the timer's [`Timer::wait_one_frame`] is
    /// called — that pause is **not** included in the measurement.
    ///
    /// This variant is designed for GPU / WebGL benchmarks where giving the
    /// compositor a full frame between renders prevents pipeline overlap from
    /// skewing results. On native the frame wait is a no-op, so the only
    /// difference from [`Self::measure`] is the per-iteration timing overhead
    /// (negligible for GPU-bound work).
    fn measure_per_iteration_with_frame_wait<F, T: Timer>(
        timer: &T,
        bench_id: &str,
        mut f: F,
        total_iters: usize,
    ) -> Statistics
    where
        F: FnMut(),
    {
        let emit_marks = total_iters <= MAX_MARKED_ITERS;
        let mut total_ns = 0.0;

        for i in 0..total_iters {
            if emit_marks {
                timer.mark(&format!("bench:{bench_id}:iter:{i}"));
            }

            let iter_start = timer.now();
            f();
            total_ns += timer.elapsed_ns(iter_start);

            if emit_marks {
                timer.mark(&format!("bench:{bench_id}:iter:{i}:end"));
                timer.measure_span(
                    &format!("{bench_id} iter {i}"),
                    &format!("bench:{bench_id}:iter:{i}"),
                    &format!("bench:{bench_id}:iter:{i}:end"),
                );
            }

            // Untimed frame wait — gives the GPU time to fully flush.
            if i + 1 < total_iters {
                timer.wait_one_frame();
            }
        }

        Statistics::from_measurement(total_ns, total_iters)
    }

    /// Run a benchmark using the provided timer, with optional callback after
    /// calibration.
    ///
    /// When `per_iteration` is `true` the measurement phase uses
    /// [`Self::measure_per_iteration`] (individual timing + frame waits);
    /// otherwise it uses the bulk [`Self::measure`] loop.
    fn run_with_timer<F, T: Timer, C: FnOnce()>(
        &self,
        timer: &T,
        id: &str,
        category: &str,
        name: &str,
        simd_variant: &str,
        mut f: F,
        on_calibrated: C,
        per_iteration: bool,
    ) -> BenchmarkResult
    where
        F: FnMut(),
    {
        // Clear stale marks/measures from any previous benchmark run.
        timer.clear_marks();
        timer.clear_measures();

        timer.mark(&format!("bench:{id}:warmup:start"));
        self.warmup(&mut f);
        timer.mark(&format!("bench:{id}:warmup:end"));
        timer.measure_span(
            &format!("{id} warm-up"),
            &format!("bench:{id}:warmup:start"),
            &format!("bench:{id}:warmup:end"),
        );

        on_calibrated();

        let total_iters = self.iterations as usize;

        timer.mark(&format!("bench:{id}:measure:start"));
        let statistics = if per_iteration {
            Self::measure_per_iteration_with_frame_wait(timer, id, f, total_iters)
        } else {
            Self::measure(timer, f, total_iters)
        };
        timer.mark(&format!("bench:{id}:measure:end"));
        timer.measure_span(
            &format!("{id} measurement"),
            &format!("bench:{id}:measure:start"),
            &format!("bench:{id}:measure:end"),
        );

        BenchmarkResult {
            id: id.to_string(),
            category: category.to_string(),
            name: name.to_string(),
            simd_variant: simd_variant.to_string(),
            statistics,
            timestamp_ms: timer.timestamp_ms(),
        }
    }

    /// Run a benchmark and return the result.
    pub fn run<F>(&self, id: &str, category: &str, name: &str, simd_variant: &str, f: F) -> BenchmarkResult
    where
        F: FnMut(),
    {
        self.run_with_timer(&PlatformTimer::default(), id, category, name, simd_variant, f, || {}, false)
    }

    /// Run a benchmark with a callback when calibration completes.
    pub fn run_with_callback<F, C>(&self, id: &str, category: &str, name: &str, simd_variant: &str, f: F, on_calibrated: C) -> BenchmarkResult
    where
        F: FnMut(),
        C: FnOnce(),
    {
        self.run_with_timer(&PlatformTimer::default(), id, category, name, simd_variant, f, on_calibrated, false)
    }

    /// Run a benchmark with per-iteration timing and an untimed frame wait
    /// between iterations.
    ///
    /// Designed for GPU / WebGL benchmarks where each iteration should be
    /// isolated by a full display-frame gap so the GPU pipeline can flush
    /// completely. The wait time (~16 ms on WASM, no-op on native) is
    /// **excluded** from the reported measurement.
    ///
    /// Note: because a ~16 ms pause is inserted between every iteration, the
    /// wall-clock duration of the benchmark will be significantly longer than
    /// the sum of iteration times alone. For example, 50 iterations adds
    /// ~800 ms of untimed waiting on top of the actual render time.
    pub fn run_with_frame_wait<F>(&self, id: &str, category: &str, name: &str, simd_variant: &str, f: F) -> BenchmarkResult
    where
        F: FnMut(),
    {
        self.run_with_timer(&PlatformTimer::default(), id, category, name, simd_variant, f, || {}, true)
    }
}

/// Timer abstraction for platform-independent benchmarking.
trait Timer {
    type Instant: Copy;

    fn now(&self) -> Self::Instant;
    fn elapsed_ns(&self, start: Self::Instant) -> f64;
    fn timestamp_ms(&self) -> u64;

    /// Record a named performance mark. No-op on native.
    fn mark(&self, _name: &str) {}

    /// Record a named measure span between two previously recorded marks.
    /// No-op on native.
    fn measure_span(&self, _name: &str, _start_mark: &str, _end_mark: &str) {}

    /// Clear all previously recorded marks. No-op on native.
    fn clear_marks(&self) {}

    /// Clear all previously recorded measures. No-op on native.
    fn clear_measures(&self) {}

    /// Busy-wait for approximately one display frame (~16 ms). Called between
    /// measurement iterations when per-iteration timing is active. The wait is
    /// **not** included in benchmark timing — it gives the GPU compositor time
    /// to fully flush between frames. No-op on native.
    fn wait_one_frame(&self) {}
}

#[cfg(not(target_arch = "wasm32"))]
type PlatformTimer = NativeTimer;
#[cfg(target_arch = "wasm32")]
type PlatformTimer = WasmTimer;

/// Native timer using std::time.
#[cfg(not(target_arch = "wasm32"))]
struct NativeTimer;

#[cfg(not(target_arch = "wasm32"))]
impl Default for NativeTimer {
    fn default() -> Self { Self }
}

#[cfg(not(target_arch = "wasm32"))]
impl Timer for NativeTimer {
    type Instant = std::time::Instant;

    fn now(&self) -> Self::Instant {
        std::time::Instant::now()
    }

    fn elapsed_ns(&self, start: Self::Instant) -> f64 {
        start.elapsed().as_nanos() as f64
    }

    fn timestamp_ms(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

/// WASM timer using Performance API.
/// Works in both Window and Worker contexts.
#[cfg(target_arch = "wasm32")]
struct WasmTimer {
    performance: web_sys::Performance,
}

#[cfg(target_arch = "wasm32")]
impl WasmTimer {
    fn new() -> Self {
        use wasm_bindgen::JsCast;

        // Use js_sys::global() which works in both Window and Worker contexts
        let global = js_sys::global();
        let performance = js_sys::Reflect::get(&global, &wasm_bindgen::JsValue::from_str("performance"))
            .expect("no performance on global")
            .unchecked_into::<web_sys::Performance>();

        Self { performance }
    }
}

#[cfg(target_arch = "wasm32")]
impl Default for WasmTimer {
    fn default() -> Self { Self::new() }
}

#[cfg(target_arch = "wasm32")]
impl Timer for WasmTimer {
    type Instant = f64; // performance.now() returns milliseconds as f64

    fn now(&self) -> Self::Instant {
        self.performance.now()
    }

    fn elapsed_ns(&self, start: Self::Instant) -> f64 {
        (self.performance.now() - start) * 1_000_000.0
    }

    fn timestamp_ms(&self) -> u64 {
        js_sys::Date::now() as u64
    }

    fn mark(&self, name: &str) {
        let _ = self.performance.mark(name);
    }

    fn measure_span(&self, name: &str, start_mark: &str, end_mark: &str) {
        let _ = self
            .performance
            .measure_with_start_mark_and_end_mark(name, start_mark, end_mark);
    }

    fn clear_marks(&self) {
        let _ = self.performance.clear_marks();
    }

    fn clear_measures(&self) {
        let _ = self.performance.clear_measures();
    }

    fn wait_one_frame(&self) {
        /// Duration in milliseconds to busy-wait between measurement iterations when
        /// per-iteration frame-wait timing is active. Approximates one display frame
        /// at 60 Hz, giving the GPU compositor time to fully flush between frames.
        /// 
        /// Without idling the CPU like this, we can enter a state where we continually
        /// flush commands to the GPU causing pipeline stalls. Pipeline stalling can mask
        /// regressions in CPU performance.
        #[cfg(target_arch = "wasm32")]
        const FRAME_WAIT_MS: f64 = 16.67;

        let target = self.performance.now() + FRAME_WAIT_MS;
        while self.performance.now() < target {}
    }
}
