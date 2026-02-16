//! Screenshot rendering for visual verification of scenes.
//!
//! Each function delegates to the corresponding renderer struct from the
//! benchmark modules, ensuring that screenshots use the exact same codepath
//! as the benchmarks.

use crate::benchmarks::scene_cpu::CpuSceneRenderer;
use crate::renderer::Renderer;
use crate::scenes::get_scenes;
use crate::vello_scenes::{draw_scene, get_vello_scenes, setup_scene};
use fearless_simd::Level;
use vello_cpu::RenderMode;

/// The result of rendering a scene screenshot.
pub struct ScreenshotResult {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Non-premultiplied RGBA8 pixel data, row-major order (4 bytes per pixel).
    pub rgba: Vec<u8>,
}

/// Render a scene by name using the Vello CPU renderer and return the pixel data.
///
/// `level` selects the SIMD instruction set; use `Level::new()` for auto-detect.
pub fn render_scene_cpu(scene_name: &str, level: Level) -> Option<ScreenshotResult> {
    let scenes = get_scenes();
    let item = scenes.iter().find(|s| s.name == scene_name)?;

    let mut renderer = CpuSceneRenderer::new(item, level);
    renderer.render_frame();

    Some(ScreenshotResult {
        width: item.width as u32,
        height: item.height as u32,
        rgba: renderer.into_rgba(),
    })
}

/// Render a scene by name using the Vello Hybrid renderer (headless wgpu)
/// and return the pixel data.
///
/// On WASM this returns `None` — hybrid screenshots are handled by
/// `vello_bench_wasm` via WebGL canvas.
pub fn render_scene_hybrid(scene_name: &str) -> Option<ScreenshotResult> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use crate::benchmarks::scene_hybrid::HybridSceneRenderer;

        let scenes = get_scenes();
        let item = scenes.iter().find(|s| s.name == scene_name)?;

        let renderer = HybridSceneRenderer::new(item);

        Some(ScreenshotResult {
            width: item.width as u32,
            height: item.height as u32,
            rgba: renderer.into_rgba(),
        })
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = scene_name;
        None
    }
}

/// Render a scene by name using the Skia CPU renderer and return the pixel data.
///
/// On WASM this returns `None` — Skia is not available on the WASM target.
pub fn render_scene_skia(scene_name: &str) -> Option<ScreenshotResult> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use crate::benchmarks::scene_skia::SkiaSceneRenderer;

        let scenes = get_scenes();
        let item = scenes.iter().find(|s| s.name == scene_name)?;

        let mut renderer = SkiaSceneRenderer::new(item);
        renderer.render_frame();

        Some(ScreenshotResult {
            width: item.width as u32,
            height: item.height as u32,
            rgba: renderer.into_rgba(),
        })
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = scene_name;
        None
    }
}

// ---------------------------------------------------------------------------
// Programmatic vello scenes (Renderer trait based)
// ---------------------------------------------------------------------------

/// Render a programmatic vello scene using the CPU backend.
pub fn render_vello_scene_cpu(scene_name: &str, level: Level) -> Option<ScreenshotResult> {
    let scenes = get_vello_scenes();
    let info = scenes.iter().find(|s| s.name == scene_name)?;

    let mut ctx: vello_cpu::RenderContext =
        Renderer::new(info.width, info.height, 0, level, RenderMode::default());
    let mut pixmap = vello_cpu::Pixmap::new(info.width, info.height);

    let state = setup_scene(scene_name, &mut ctx).expect("scene not found");
    draw_scene(scene_name, state.as_ref(), &mut ctx);
    ctx.flush();
    ctx.render_to_pixmap(&mut pixmap);

    let rgba = pixmap
        .take_unpremultiplied()
        .into_iter()
        .flat_map(|p| [p.r, p.g, p.b, p.a])
        .collect();

    Some(ScreenshotResult {
        width: info.width as u32,
        height: info.height as u32,
        rgba,
    })
}

/// Render a programmatic vello scene using the Hybrid (wgpu) backend.
///
/// On WASM this returns `None` — hybrid screenshots are handled by
/// `vello_bench_wasm` via WebGL canvas.
pub fn render_vello_scene_hybrid(scene_name: &str) -> Option<ScreenshotResult> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use crate::renderer::HybridRenderer;

        let scenes = get_vello_scenes();
        let info = scenes.iter().find(|s| s.name == scene_name)?;

        let mut hybrid: HybridRenderer =
            Renderer::new(info.width, info.height, 0, Level::new(), RenderMode::default());
        let mut pixmap = vello_cpu::Pixmap::new(info.width, info.height);

        let state = setup_scene(scene_name, &mut hybrid).expect("scene not found");
        draw_scene(scene_name, state.as_ref(), &mut hybrid);
        hybrid.render_to_pixmap(&mut pixmap);

        let rgba = pixmap
            .take_unpremultiplied()
            .into_iter()
            .flat_map(|p| [p.r, p.g, p.b, p.a])
            .collect();

        Some(ScreenshotResult {
            width: info.width as u32,
            height: info.height as u32,
            rgba,
        })
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = scene_name;
        None
    }
}
