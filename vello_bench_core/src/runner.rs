use crate::result::{BenchmarkResult, Statistics};

#[derive(Debug, Clone)]
pub struct BenchRunner {
    pub calibration_ms: u64,
    pub measurement_ms: u64,
}

impl BenchRunner {
    pub fn new(calibration_ms: u64, measurement_ms: u64) -> Self {
        Self { calibration_ms, measurement_ms }
    }
}

impl BenchRunner {
    fn calibrate<F, T: Timer>(&self, timer: &T, mut f: F) -> (usize, f64)
    where
        F: FnMut(),
    {
        let target_ns = self.calibration_ms as f64 * 1_000_000.0;
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
        let (batch_size, batch_time_ns) = self.calibrate(timer, &mut f);

        on_calibrated();

        let target_ns = self.measurement_ms as f64 * 1_000_000.0;
        let iters_per_ns = batch_size as f64 / batch_time_ns;
        let total_iters = (iters_per_ns * target_ns).ceil() as usize;

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
    pub fn run<F>(&self, id: &str, category: &str, name: &str, simd_variant: &str, f: F) -> BenchmarkResult
    where
        F: FnMut(),
    {
        self.run_with_timer(&PlatformTimer::default(), id, category, name, simd_variant, f, || {})
    }

    /// Run a benchmark with a callback when calibration completes.
    pub fn run_with_callback<F, C>(&self, id: &str, category: &str, name: &str, simd_variant: &str, f: F, on_calibrated: C) -> BenchmarkResult
    where
        F: FnMut(),
        C: FnOnce(),
    {
        self.run_with_timer(&PlatformTimer::default(), id, category, name, simd_variant, f, on_calibrated)
    }
}

/// Timer abstraction for platform-independent benchmarking.
trait Timer {
    type Instant: Copy;

    fn now(&self) -> Self::Instant;
    fn elapsed_ns(&self, start: Self::Instant) -> f64;
    fn timestamp_ms(&self) -> u64;
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

// Note that the JS performance timer is likely not super accurate. However,
// in our case we have a calibration phase to estimate how many iterations are needed
// to run for a given number of seconds, and then we run the benchmarks all at once
// for the estimated number of iterations. Therefore, being a few milliseconds off
// does not matter that much.
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
        self.performance.now() as u64
    }
}
