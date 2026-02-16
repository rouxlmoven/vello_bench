//! Benchmarks that run programmatic vello scenes using the Vello CPU backend.
//!
//! Each scene registered in `vello_scenes` becomes a benchmark under the
//! `vello_cpu` category. The benchmark measures: scene draw + flush +
//! rasterisation to a `Pixmap`. Image uploads happen during setup (not timed).

use crate::registry::BenchmarkInfo;
use crate::renderer::Renderer;
use crate::result::BenchmarkResult;
use crate::runner::BenchRunner;
use crate::simd::level_suffix;
use crate::vello_scenes::{draw_scene, get_vello_scenes, setup_scene};
use fearless_simd::Level;
use vello_cpu::{Pixmap, RenderContext, RenderMode};

const CATEGORY: &str = "vello_cpu";

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

pub fn run(name: &str, runner: &BenchRunner, level: Level) -> Option<BenchmarkResult> {
    let scenes = get_vello_scenes();
    let info = scenes.iter().find(|s| s.name == name)?;
    let simd_variant = level_suffix(level);

    let mut ctx: RenderContext =
        Renderer::new(info.width, info.height, 0, level, RenderMode::default());
    let mut pixmap = Pixmap::new(info.width, info.height);

    // Setup phase — image uploads etc. (not timed).
    let state = setup_scene(name, &mut ctx).expect("scene not found in setup");

    Some(runner.run(
        &format!("{CATEGORY}/{name}"),
        CATEGORY,
        name,
        simd_variant,
        #[inline(always)]
        || {
            draw_scene(name, state.as_ref(), &mut ctx);
            ctx.flush();
            ctx.render_to_pixmap(&mut pixmap);
            std::hint::black_box(&pixmap);
        },
    ))
}
