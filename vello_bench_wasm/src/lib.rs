//! WASM bindings for vello benchmarks.

#![allow(missing_docs, reason = "Not needed for benchmarks")]
#![cfg(target_arch = "wasm32")]

use vello_bench_core::{BenchRunner, available_level_infos};
use wasm_bindgen::prelude::*;

/// Initialize the WASM module.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// List all available benchmarks.
#[wasm_bindgen]
pub fn list_benchmarks() -> JsValue {
    let benchmarks = vello_bench_core::get_benchmark_list();
    serde_wasm_bindgen::to_value(&benchmarks).unwrap()
}

/// Get available SIMD levels for this platform.
#[wasm_bindgen]
pub fn get_simd_levels() -> JsValue {
    let level_info = available_level_infos();
    serde_wasm_bindgen::to_value(&level_info).unwrap()
}

/// Check if SIMD128 is available.
#[wasm_bindgen]
pub fn has_simd128() -> bool {
    #[cfg(target_feature = "simd128")]
    {
        true
    }
    #[cfg(not(target_feature = "simd128"))]
    {
        false
    }
}

/// Run a single benchmark by ID.
#[wasm_bindgen]
pub fn run_benchmark(id: &str, calibration_ms: u32, measurement_ms: u32) -> JsValue {
    use fearless_simd::Level;

    let runner = BenchRunner::new(calibration_ms.into(), measurement_ms.into());
    let level = Level::new();

    match vello_bench_core::run_benchmark_by_id(&runner, id, level) {
        Some(result) => serde_wasm_bindgen::to_value(&result).unwrap(),
        None => JsValue::NULL,
    }
}

// ---------------------------------------------------------------------------
// Hybrid WebGL benchmarks — run on the main thread, not in a Web Worker
// ---------------------------------------------------------------------------

use std::cell::RefCell;

use anyrender_vello_hybrid::{WebGlRenderContext, WebGlScenePainter};

thread_local! {
    static HYBRID_STATE: RefCell<Option<HybridState>> = const { RefCell::new(None) };
}

struct HybridState {
    renderer: vello_hybrid::WebGlRenderer,
    canvas: web_sys::HtmlCanvasElement,
}

/// Initialize the hybrid WebGL renderer with a canvas element.
/// Called from the main thread. The canvas can be hidden / off-screen.
#[wasm_bindgen]
pub fn init_hybrid(canvas: web_sys::HtmlCanvasElement) -> bool {
    let renderer = vello_hybrid::WebGlRenderer::new(&canvas);
    HYBRID_STATE.with(|s| {
        *s.borrow_mut() = Some(HybridState { renderer, canvas });
    });
    true
}

/// Deserialize the scene with a [`WebGlRenderContext`], registering images
/// directly in the WebGL backend format. Pending GPU uploads will be flushed
/// lazily by the scene painter on first use.
fn deserialize_scene_webgl(
    item: &vello_bench_core::scenes::SceneItem,
) -> (anyrender::Scene, WebGlRenderContext) {
    let mut ctx = WebGlRenderContext::new();
    let scene = item
        .archive
        .to_scene(&mut ctx)
        .expect("Failed to deserialize scene for WebGL backend");
    (scene, ctx)
}

// ---------------------------------------------------------------------------
// Screenshots — render a scene once and return pixel data for verification
// ---------------------------------------------------------------------------

/// Render a scene via the CPU renderer and return the pixel data.
/// Returns a JS object `{ width, height, data: Uint8ClampedArray }` with
/// non-premultiplied RGBA8 pixels, compatible with `ImageData`.
#[wasm_bindgen]
pub fn screenshot_cpu(scene_name: &str) -> JsValue {
    let result = match vello_bench_core::screenshot::render_scene_cpu(
        scene_name,
        fearless_simd::Level::new(),
    ) {
        Some(r) => r,
        None => return JsValue::NULL,
    };

    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"width".into(), &result.width.into()).unwrap();
    js_sys::Reflect::set(&obj, &"height".into(), &result.height.into()).unwrap();

    let clamped = js_sys::Uint8ClampedArray::from(result.rgba.as_slice());
    js_sys::Reflect::set(&obj, &"data".into(), &clamped).unwrap();

    obj.into()
}

/// Ensure the canvas matches the requested dimensions.
///
/// If a resize is needed, `set_width`/`set_height` resets the WebGL context,
/// invalidating all compiled shaders and uploaded textures. In that case we
/// re-create the [`WebGlRenderer`] so it picks up the fresh GL context.
fn ensure_canvas_size(state: &mut HybridState, width: u32, height: u32) {
    if state.canvas.width() != width || state.canvas.height() != height {
        state.canvas.set_width(width);
        state.canvas.set_height(height);
        state.renderer = vello_hybrid::WebGlRenderer::new(&state.canvas);
    }
}

/// Render a single hybrid frame: build the scene, render via WebGL, and sync.
///
/// Shared by both `render_hybrid_once` (screenshot) and `run_hybrid_benchmark`
/// (hot loop) to ensure the same codepath.
fn render_hybrid_frame(
    renderer: &mut vello_hybrid::WebGlRenderer,
    ctx: &mut WebGlRenderContext,
    scene: &anyrender::Scene,
    hybrid_scene: &mut vello_hybrid::Scene,
    render_size: &vello_hybrid::RenderSize,
) {
    use anyrender::PaintScene;
    use vello_common::kurbo::Affine;

    {
        let mut painter = WebGlScenePainter::new(ctx, renderer, hybrid_scene);
        painter.append_scene(scene.clone(), Affine::IDENTITY);
    }

    renderer
        .render(hybrid_scene, render_size)
        .expect("WebGL render failed");

    renderer.gl_context().finish();
    hybrid_scene.reset();
}

/// Render a scene once via the WebGL hybrid renderer.
/// After calling this, the hybrid canvas contains the rendered output.
/// The JS side can then use `canvas.toDataURL()` to capture a screenshot.
/// Returns true if rendering succeeded, false otherwise.
#[wasm_bindgen]
pub fn render_hybrid_once(scene_name: &str) -> bool {
    let scenes = vello_bench_core::scenes::get_scenes();
    let item = match scenes.iter().find(|s| s.name == scene_name) {
        Some(item) => item,
        None => return false,
    };

    let width = item.width as u32;
    let height = item.height as u32;

    HYBRID_STATE.with(|state_cell| {
        let mut state_opt = state_cell.borrow_mut();
        let state = match state_opt.as_mut() {
            Some(s) => s,
            None => return false,
        };

        ensure_canvas_size(state, width, height);

        let (scene, mut ctx) = deserialize_scene_webgl(item);

        let render_size = vello_hybrid::RenderSize { width, height };
        let mut hybrid_scene = vello_hybrid::Scene::new(item.width, item.height);

        render_hybrid_frame(
            &mut state.renderer,
            &mut ctx,
            &scene,
            &mut hybrid_scene,
            &render_size,
        );

        true
    })
}

/// Run a hybrid scene benchmark on the main thread using WebGL.
/// Returns the benchmark result as a JsValue, or null if the benchmark
/// was not found or hybrid is not initialized.
#[wasm_bindgen]
pub fn run_hybrid_benchmark(id: &str, calibration_ms: u32, measurement_ms: u32) -> JsValue {
    // Only handle scene_hybrid/ benchmarks
    let scene_name = match id.strip_prefix("scene_hybrid/") {
        Some(name) => name,
        None => return JsValue::NULL,
    };

    let scenes = vello_bench_core::scenes::get_scenes();
    let item = match scenes.iter().find(|s| s.name == scene_name) {
        Some(item) => item,
        None => return JsValue::NULL,
    };

    let width = item.width as u32;
    let height = item.height as u32;

    HYBRID_STATE.with(|state_cell| {
        let mut state_opt = state_cell.borrow_mut();
        let state = match state_opt.as_mut() {
            Some(s) => s,
            None => return JsValue::NULL,
        };

        ensure_canvas_size(state, width, height);

        let (scene, mut ctx) = deserialize_scene_webgl(item);

        let render_size = vello_hybrid::RenderSize { width, height };
        let mut hybrid_scene = vello_hybrid::Scene::new(item.width, item.height);

        let runner = BenchRunner::new(calibration_ms.into(), measurement_ms.into());
        let simd_variant = vello_bench_core::simd::level_suffix(fearless_simd::Level::new());

        let result = runner.run(
            id,
            "scene_hybrid",
            scene_name,
            simd_variant,
            #[inline(always)]
            || {
                render_hybrid_frame(
                    &mut state.renderer,
                    &mut ctx,
                    &scene,
                    &mut hybrid_scene,
                    &render_size,
                );
            },
        );

        serde_wasm_bindgen::to_value(&result).unwrap()
    })
}

// ---------------------------------------------------------------------------
// WebGL HybridRenderer — implements vello_bench_core::renderer::Renderer
// for programmatic vello scene benchmarks on WASM.
// ---------------------------------------------------------------------------

mod webgl_renderer;

// ---------------------------------------------------------------------------
// Programmatic vello scene benchmarks / screenshots — WebGL hybrid backend
// ---------------------------------------------------------------------------

use vello_bench_core::vello_scenes::{draw_scene, get_vello_scenes, setup_scene};

/// Run a programmatic vello scene benchmark via the WebGL hybrid renderer.
/// Returns the benchmark result as a JsValue, or null if not found.
#[wasm_bindgen]
pub fn run_vello_hybrid_benchmark(
    id: &str,
    calibration_ms: u32,
    measurement_ms: u32,
) -> JsValue {
    let scene_name = match id.strip_prefix("vello_hybrid/") {
        Some(name) => name,
        None => return JsValue::NULL,
    };

    let scenes = get_vello_scenes();
    let info = match scenes.iter().find(|s| s.name == scene_name) {
        Some(info) => info,
        None => return JsValue::NULL,
    };

    HYBRID_STATE.with(|state_cell| {
        let mut state_opt = state_cell.borrow_mut();
        let state = match state_opt.as_mut() {
            Some(s) => s,
            None => return JsValue::NULL,
        };

        ensure_canvas_size(state, info.width.into(), info.height.into());

        let mut hybrid = webgl_renderer::WebGlHybridRenderer::from_state(
            info.width,
            info.height,
            &mut state.renderer,
        );

        // Setup phase — image uploads etc. (not timed).
        let scene_state =
            setup_scene(scene_name, &mut hybrid).expect("vello scene not found in setup");

        let runner = BenchRunner::new(calibration_ms.into(), measurement_ms.into());
        let simd_variant = vello_bench_core::simd::level_suffix(fearless_simd::Level::new());

        let result = runner.run(
            id,
            "vello_hybrid",
            scene_name,
            simd_variant,
            #[inline(always)]
            || {
                draw_scene(scene_name, scene_state.as_ref(), &mut hybrid);
                hybrid.render_and_sync();
            },
        );

        serde_wasm_bindgen::to_value(&result).unwrap()
    })
}

/// Render a programmatic vello scene once via the WebGL hybrid renderer.
/// After calling this, the hybrid canvas contains the rendered output.
/// Returns true on success.
#[wasm_bindgen]
pub fn render_vello_hybrid_once(scene_name: &str) -> bool {
    let scenes = get_vello_scenes();
    let info = match scenes.iter().find(|s| s.name == scene_name) {
        Some(info) => info,
        None => return false,
    };

    HYBRID_STATE.with(|state_cell| {
        let mut state_opt = state_cell.borrow_mut();
        let state = match state_opt.as_mut() {
            Some(s) => s,
            None => return false,
        };

        ensure_canvas_size(state, info.width.into(), info.height.into());

        let mut hybrid = webgl_renderer::WebGlHybridRenderer::from_state(
            info.width,
            info.height,
            &mut state.renderer,
        );

        let scene_state =
            setup_scene(scene_name, &mut hybrid).expect("vello scene not found");
        draw_scene(scene_name, scene_state.as_ref(), &mut hybrid);
        hybrid.render_and_sync();

        true
    })
}

/// Render a programmatic vello scene via CPU and return pixel data.
/// Returns a JS object `{ width, height, data: Uint8ClampedArray }`.
#[wasm_bindgen]
pub fn screenshot_vello_cpu(scene_name: &str) -> JsValue {
    let result = match vello_bench_core::screenshot::render_vello_scene_cpu(
        scene_name,
        fearless_simd::Level::new(),
    ) {
        Some(r) => r,
        None => return JsValue::NULL,
    };

    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"width".into(), &result.width.into()).unwrap();
    js_sys::Reflect::set(&obj, &"height".into(), &result.height.into()).unwrap();

    let clamped = js_sys::Uint8ClampedArray::from(result.rgba.as_slice());
    js_sys::Reflect::set(&obj, &"data".into(), &clamped).unwrap();

    obj.into()
}
