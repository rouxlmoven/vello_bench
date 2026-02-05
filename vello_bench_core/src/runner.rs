use crate::result::{BenchmarkResult, Statistics};

/// Configuration for benchmark runs.
#[derive(Debug, Clone)]
pub struct BenchRunner {
    /// Measurement duration in milliseconds.
    pub measurement_ms: u64,
}

impl BenchRunner {
    /// Create a new runner with custom measurement time.
    pub fn new(_warmup_ms: u64, measurement_ms: u64) -> Self {
        // warmup_ms is ignored - calibration handles warmup
        Self { measurement_ms }
    }

    /// Create a runner with default timing (5s measurement).
    pub fn default_timing() -> Self {
        Self {
            measurement_ms: 5000,
        }
    }

    /// Calibrate to find iteration count that takes ~500ms.
    /// Returns (batch_size, batch_time_ns).
    fn calibrate<F, T: Timer>(timer: &T, mut f: F) -> (usize, f64)
    where
        F: FnMut(),
    {
        let target_ns = 1_500_000_000.0; // 1500ms in nanoseconds
        let mut batch_size = 1usize;

        loop {
            let start = timer.now();
            for _ in 0..batch_size {
                f();
            }
            let elapsed_ns = timer.elapsed_ns(start);

            if elapsed_ns >= target_ns {
                return (batch_size, elapsed_ns);
            }

            batch_size *= 2;
        }
    }

    /// Run the measurement phase and return statistics.
    fn measure<F, T: Timer>(timer: &T, mut f: F, total_iters: usize) -> Statistics
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

    /// Run a benchmark using the provided timer, with optional callback after calibration.
    fn run_with_timer<F, T: Timer, C: FnOnce()>(
        &self,
        timer: &T,
        id: &str,
        category: &str,
        name: &str,
        simd_variant: &str,
        mut f: F,
        on_calibrated: C,
    ) -> BenchmarkResult
    where
        F: FnMut(),
    {
        // Calibration phase: find batch size that takes ~500ms
        let (batch_size, batch_time_ns) = Self::calibrate(timer, &mut f);

        // Notify that calibration is complete
        on_calibrated();

        // Calculate iterations needed for target measurement time
        let target_ns = self.measurement_ms as f64 * 1_000_000.0;
        let iters_per_ns = batch_size as f64 / batch_time_ns;
        let total_iters = (iters_per_ns * target_ns).ceil() as usize;

        // Single measurement
        let statistics = Self::measure(timer, f, total_iters);

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
    #[cfg(not(target_arch = "wasm32"))]
    pub fn  run<F>(&self, id: &str, category: &str, name: &str, simd_variant: &str, f: F) -> BenchmarkResult
    where
        F: FnMut(),
    {
        self.run_with_timer(&NativeTimer, id, category, name, simd_variant, f, || {})
    }

    /// Run a benchmark with a callback when calibration completes.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_with_callback<F, C>(&self, id: &str, category: &str, name: &str, simd_variant: &str, f: F, on_calibrated: C) -> BenchmarkResult
    where
        F: FnMut(),
        C: FnOnce(),
    {
        self.run_with_timer(&NativeTimer, id, category, name, simd_variant, f, on_calibrated)
    }

    /// Run a benchmark and return the result (WASM version).
    #[cfg(target_arch = "wasm32")]
    pub fn run<F>(&self, id: &str, category: &str, name: &str, simd_variant: &str, f: F) -> BenchmarkResult
    where
        F: FnMut(),
    {
        self.run_with_timer(&WasmTimer::new(), id, category, name, simd_variant, f, || {})
    }

    /// Run a benchmark with a callback when calibration completes (WASM version).
    #[cfg(target_arch = "wasm32")]
    pub fn run_with_callback<F, C>(&self, id: &str, category: &str, name: &str, simd_variant: &str, f: F, on_calibrated: C) -> BenchmarkResult
    where
        F: FnMut(),
        C: FnOnce(),
    {
        self.run_with_timer(&WasmTimer::new(), id, category, name, simd_variant, f, on_calibrated)
    }
}

impl Default for BenchRunner {
    fn default() -> Self {
        Self::default_timing()
    }
}

/// Timer abstraction for platform-independent benchmarking.
trait Timer {
    type Instant: Copy;

    fn now(&self) -> Self::Instant;
    fn elapsed_ns(&self, start: Self::Instant) -> f64;
    fn timestamp_ms(&self) -> u64;
}

/// Native timer using std::time.
#[cfg(not(target_arch = "wasm32"))]
struct NativeTimer;

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
impl Timer for WasmTimer {
    type Instant = f64; // performance.now() returns milliseconds as f64

    fn now(&self) -> Self::Instant {
        self.performance.now()
    }

    fn elapsed_ns(&self, start: Self::Instant) -> f64 {
        (self.performance.now() - start) * 1_000_000.0
    }

    fn timestamp_ms(&self) -> u64 {
        self.performance.now() as u64
    }
}
