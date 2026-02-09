//! Benchmarks that replay serialized AnyRender scenes using Skia (CPU rasterizer).
//!
//! On native: uses `anyrender_skia::SkiaImageRenderer` for rasterization.
//! On WASM: Skia is not available, so `run()` returns `None`.
//!
//! Each scene in the `scenes/` directory becomes a benchmark under the
//! `scene_skia` category. The benchmark measures the full rendering pipeline:
//! scene replay (via `SkiaScenePainter`) + Skia CPU rasterization.

use crate::registry::BenchmarkInfo;
use crate::result::BenchmarkResult;
use crate::runner::BenchRunner;
use crate::scenes::get_scenes;
use fearless_simd::Level;

const CATEGORY: &str = "scene_skia";

/// Encapsulates all state needed to render a scene with the Skia backend.
///
/// Used by both benchmarks (hot loop) and screenshots (single render) to
/// ensure the exact same codepath. Native-only — Skia is not available on WASM.
#[cfg(not(target_arch = "wasm32"))]
pub struct SkiaSceneRenderer {
    ctx: anyrender_skia::SkiaRenderContext,
    renderer: anyrender_skia::SkiaImageRenderer,
    buffer: Vec<u8>,
    scene: anyrender::Scene,
}

#[cfg(not(target_arch = "wasm32"))]
impl SkiaSceneRenderer {
    /// Set up a Skia renderer for the given scene.
    pub fn new(item: &crate::scenes::SceneItem) -> Self {
        use anyrender::ImageRenderer;

        let width = item.width as u32;
        let height = item.height as u32;
        let buffer = vec![0u8; (width * height * 4) as usize];
        let renderer = anyrender_skia::SkiaImageRenderer::new(width, height);

        let mut ctx = anyrender_skia::SkiaRenderContext::new();
        let scene = item
            .archive
            .to_scene(&mut ctx)
            .expect("Failed to deserialize scene for Skia backend");

        Self {
            ctx,
            renderer,
            buffer,
            scene,
        }
    }

    /// Render one frame. This is the benchmarked operation.
    #[inline(always)]
    pub fn render_frame(&mut self) {
        use anyrender::ImageRenderer;
        use anyrender::PaintScene;
        use vello_common::kurbo::Affine;

        self.renderer.render(
            &mut self.ctx,
            |painter| {
                painter.append_scene(self.scene.clone(), Affine::IDENTITY);
            },
            &mut self.buffer,
        );
    }

    /// Consume the renderer and return the RGBA8 pixel data.
    pub fn into_rgba(self) -> Vec<u8> {
        self.buffer
    }
}

pub fn list() -> Vec<BenchmarkInfo> {
    get_scenes()
        .iter()
        .map(|item| BenchmarkInfo {
            id: format!("{CATEGORY}/{}", item.name),
            category: CATEGORY.into(),
            name: item.name.clone(),
        })
        .collect()
}

/// Run a Skia benchmark. On WASM this always returns `None` because
/// Skia (skia-safe) is not available on the WASM target.
pub fn run(name: &str, runner: &BenchRunner, _level: Level) -> Option<BenchmarkResult> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        run_native(name, runner)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (name, runner);
        None
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn run_native(name: &str, runner: &BenchRunner) -> Option<BenchmarkResult> {
    let scenes = get_scenes();
    let item = scenes.iter().find(|s| s.name == name)?;

    // Skia does not use SIMD level selection — always report "n/a".
    let simd_variant = "n/a";

    let mut renderer = SkiaSceneRenderer::new(item);

    Some(runner.run(
        &format!("{CATEGORY}/{name}"),
        CATEGORY,
        name,
        simd_variant,
        #[inline(always)]
        || {
            renderer.render_frame();
            std::hint::black_box(&renderer);
        },
    ))
}
