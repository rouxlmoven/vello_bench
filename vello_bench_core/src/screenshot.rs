//! Screenshot rendering for visual verification of AnyRender scenes.
//!
//! Each function delegates to the corresponding renderer struct from the
//! benchmark modules, ensuring that screenshots use the exact same codepath
//! as the benchmarks.

use crate::benchmarks::scene_cpu::CpuSceneRenderer;
use crate::scenes::get_scenes;
use fearless_simd::Level;

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
