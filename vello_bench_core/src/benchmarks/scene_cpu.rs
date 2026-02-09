//! Benchmarks that replay serialized AnyRender scenes using Vello CPU.
//!
//! Each scene in the `scenes/` directory becomes a benchmark under the
//! `scene_cpu` category. The benchmark measures the full rendering pipeline:
//! scene replay (via `VelloCpuScenePainter`) + rasterization to a `Pixmap`.

use crate::registry::BenchmarkInfo;
use crate::result::BenchmarkResult;
use crate::runner::BenchRunner;
use crate::scenes::{SceneItem, get_scenes};
use crate::simd::level_suffix;
use anyrender::PaintScene;
use fearless_simd::Level;
use vello_common::kurbo::Affine;
use vello_cpu::{Pixmap, RenderContext as VelloCpuRenderCtx, RenderSettings};

const CATEGORY: &str = "scene_cpu";

/// Encapsulates all state needed to render a scene with the Vello CPU backend.
///
/// Used by both benchmarks (hot loop) and screenshots (single render) to
/// ensure the exact same codepath.
pub struct CpuSceneRenderer {
    anyrender_ctx: anyrender_vello_cpu::VelloCpuRenderContext,
    render_ctx: VelloCpuRenderCtx,
    pixmap: Pixmap,
    scene: anyrender::Scene,
}

impl CpuSceneRenderer {
    /// Set up a CPU renderer for the given scene and SIMD level.
    pub fn new(item: &SceneItem, level: Level) -> Self {
        let settings = RenderSettings {
            level,
            ..Default::default()
        };
        let render_ctx = VelloCpuRenderCtx::new_with(item.width, item.height, settings);
        let pixmap = Pixmap::new(item.width, item.height);

        let mut anyrender_ctx = anyrender_vello_cpu::VelloCpuRenderContext::new();
        let scene = item
            .archive
            .to_scene(&mut anyrender_ctx)
            .expect("Failed to deserialize scene for CPU backend");

        Self {
            anyrender_ctx,
            render_ctx,
            pixmap,
            scene,
        }
    }

    /// Render one frame. This is the benchmarked operation.
    #[inline(always)]
    pub fn render_frame(&mut self) {
        {
            let mut painter = anyrender_vello_cpu::VelloCpuScenePainter::new(
                &self.anyrender_ctx,
                &mut self.render_ctx,
            );
            painter.reset();
            painter.append_scene(self.scene.clone(), Affine::IDENTITY);
        }
        self.render_ctx.flush();
        self.render_ctx.render_to_pixmap(&mut self.pixmap);
    }

    /// Consume the renderer and extract non-premultiplied RGBA8 pixel data.
    pub fn into_rgba(self) -> Vec<u8> {
        self.pixmap
            .take_unpremultiplied()
            .into_iter()
            .flat_map(|p| [p.r, p.g, p.b, p.a])
            .collect()
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

pub fn run(name: &str, runner: &BenchRunner, level: Level) -> Option<BenchmarkResult> {
    let scenes = get_scenes();
    let item = scenes.iter().find(|s| s.name == name)?;
    let simd_variant = level_suffix(level);

    let mut renderer = CpuSceneRenderer::new(item, level);

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
