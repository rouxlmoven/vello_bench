//! Benchmarks that run programmatic vello scenes using the Vello Hybrid backend.
//!
//! On native (non-WASM): uses wgpu for headless GPU rendering.
//! On WASM: hybrid benchmarks are handled by the `vello_bench_wasm` crate
//! on the main thread using WebGL (not available in this core crate).
//!
//! Each scene registered in `vello_scenes` becomes a benchmark under the
//! `vello_hybrid` category. The benchmark measures: scene draw + GPU render +
//! GPU sync. Image uploads happen during setup (not timed).

use crate::registry::BenchmarkInfo;
use crate::result::BenchmarkResult;
use crate::runner::BenchRunner;
use crate::vello_scenes::get_vello_scenes;
use fearless_simd::Level;

const CATEGORY: &str = "vello_hybrid";

pub fn list() -> Vec<BenchmarkInfo> {
    get_vello_scenes()
        .iter()
        .map(|scene| BenchmarkInfo {
            id: format!("{CATEGORY}/{}", scene.name),
            category: CATEGORY.into(),
            name: scene.name.to_string(),
        })
        .collect()
}

/// Run a hybrid benchmark. On WASM this always returns `None` because
/// hybrid WASM benchmarks are driven from JS via the `vello_bench_wasm` crate.
pub fn run(name: &str, runner: &BenchRunner, level: Level) -> Option<BenchmarkResult> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        run_native(name, runner, level)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (name, runner, level);
        None
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn run_native(name: &str, runner: &BenchRunner, level: Level) -> Option<BenchmarkResult> {
    use crate::renderer::{HybridRenderer, Renderer};
    use crate::simd::level_suffix;
    use crate::vello_scenes::{draw_scene, setup_scene};
    use vello_cpu::RenderMode;

    let scenes = get_vello_scenes();
    let info = scenes.iter().find(|s| s.name == name)?;
    let simd_variant = level_suffix(level);

    let mut hybrid: HybridRenderer =
        Renderer::new(info.width, info.height, 0, level, RenderMode::default());

    // Setup phase — image uploads etc. (not timed).
    let state = setup_scene(name, &mut hybrid).expect("scene not found in setup");

    Some(runner.run(
        &format!("{CATEGORY}/{name}"),
        CATEGORY,
        name,
        simd_variant,
        #[inline(always)]
        || {
            draw_scene(name, state.as_ref(), &mut hybrid);
            hybrid.render_and_sync();
        },
    ))
}
