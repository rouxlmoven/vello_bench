//! WebGL `HybridRenderer` implementation of the [`Renderer`] trait for WASM.
//!
//! This is adapted from `vello_sparse_tests/tests/renderer.rs` (the
//! `#[cfg(all(target_arch = "wasm32", feature = "webgl"))]` variant) and
//! rewritten as a borrowed wrapper so it can share the WebGL renderer owned
//! by the WASM module's `HybridState`.

use std::cell::RefCell;
use std::sync::Arc;

use vello_bench_core::renderer::Renderer;
use vello_common::filter_effects::Filter;
use vello_common::glyph::GlyphRunBuilder;
use vello_common::kurbo::{Affine, BezPath, Rect, Stroke};
use vello_common::mask::Mask;
use vello_common::paint::{ImageSource, PaintType};
use vello_common::peniko::{BlendMode, Fill, FontData};
use vello_common::pixmap::Pixmap;
use vello_common::recording::{Recordable, Recorder, Recording};
use vello_cpu::RenderMode;
use vello_hybrid::Scene;

/// A wrapper around a borrowed [`vello_hybrid::WebGlRenderer`] and a
/// [`Scene`] that implements [`Renderer`].
///
/// The inner renderer is held in a `RefCell` to support the `render_to_pixmap(&self)`
/// signature required by the trait (same pattern as the original `renderer.rs`).
pub(crate) struct WebGlHybridRenderer<'a> {
    scene: Scene,
    renderer: RefCell<&'a mut vello_hybrid::WebGlRenderer>,
}

impl<'a> WebGlHybridRenderer<'a> {
    /// Create a renderer that borrows the shared WebGL renderer.
    pub fn from_state(
        width: u16,
        height: u16,
        renderer: &'a mut vello_hybrid::WebGlRenderer,
    ) -> Self {
        let scene = Scene::new(width, height);
        Self {
            scene,
            renderer: RefCell::new(renderer),
        }
    }

    /// Render the current scene via WebGL and sync. Used in the benchmark
    /// hot loop (no pixel readback).
    pub fn render_and_sync(&mut self) {
        let render_size = vello_hybrid::RenderSize {
            width: self.scene.width().into(),
            height: self.scene.height().into(),
        };

        self.renderer
            .borrow_mut()
            .render(&self.scene, &render_size)
            .expect("WebGL render failed");

        self.renderer.borrow_mut().gl_context().finish();
        self.scene.reset();
    }
}

impl Renderer for WebGlHybridRenderer<'_> {
    type GlyphRenderer = Scene;

    fn new(
        _width: u16,
        _height: u16,
        _num_threads: u16,
        _level: fearless_simd::Level,
        _: RenderMode,
    ) -> Self {
        panic!(
            "WebGlHybridRenderer cannot be created via Renderer::new(); \
             use WebGlHybridRenderer::from_state() instead"
        );
    }

    fn fill_path(&mut self, path: &BezPath) {
        self.scene.fill_path(path);
    }

    fn stroke_path(&mut self, path: &BezPath) {
        self.scene.stroke_path(path);
    }

    fn fill_rect(&mut self, rect: &Rect) {
        self.scene.fill_rect(rect);
    }

    fn fill_blurred_rounded_rect(&mut self, _: &Rect, _: f32, _: f32) {
        unimplemented!()
    }

    fn stroke_rect(&mut self, rect: &Rect) {
        self.scene.stroke_rect(rect);
    }

    fn glyph_run(&mut self, font: &FontData) -> GlyphRunBuilder<'_, Self::GlyphRenderer> {
        self.scene.glyph_run(font)
    }

    fn push_layer(
        &mut self,
        clip: Option<&BezPath>,
        blend_mode: Option<BlendMode>,
        opacity: Option<f32>,
        mask: Option<Mask>,
        filter: Option<Filter>,
    ) {
        self.scene
            .push_layer(clip, blend_mode, opacity, mask, filter);
    }

    fn flush(&mut self) {}

    fn push_clip_layer(&mut self, path: &BezPath) {
        self.scene.push_clip_layer(path);
    }

    fn push_clip_path(&mut self, path: &BezPath) {
        self.scene.push_clip_path(path);
    }

    fn push_blend_layer(&mut self, mode: BlendMode) {
        self.scene.push_layer(None, Some(mode), None, None, None);
    }

    fn push_opacity_layer(&mut self, opacity: f32) {
        self.scene.push_layer(None, None, Some(opacity), None, None);
    }

    fn push_mask_layer(&mut self, _: Mask) {
        unimplemented!()
    }

    fn push_filter_layer(&mut self, filter: Filter) {
        self.scene.push_filter_layer(filter);
    }

    fn pop_layer(&mut self) {
        self.scene.pop_layer();
    }

    fn pop_clip_path(&mut self) {
        self.scene.pop_clip_path();
    }

    fn set_stroke(&mut self, stroke: Stroke) {
        self.scene.set_stroke(stroke);
    }

    fn set_mask(&mut self, _: Mask) {
        unimplemented!()
    }

    fn set_paint(&mut self, paint: impl Into<PaintType>) {
        self.scene.set_paint(paint);
    }

    fn set_paint_transform(&mut self, affine: Affine) {
        self.scene.set_paint_transform(affine);
    }

    fn set_fill_rule(&mut self, fill_rule: Fill) {
        self.scene.set_fill_rule(fill_rule);
    }

    fn set_transform(&mut self, transform: Affine) {
        self.scene.set_transform(transform);
    }

    fn set_blend_mode(&mut self, _: BlendMode) {
        unimplemented!()
    }

    fn set_aliasing_threshold(&mut self, aliasing_threshold: Option<u8>) {
        self.scene.set_aliasing_threshold(aliasing_threshold);
    }

    fn set_filter_effect(&mut self, filter: Filter) {
        self.scene.set_filter_effect(filter);
    }

    fn reset_filter_effect(&mut self) {
        self.scene.reset_filter_effect();
    }

    fn render_to_pixmap(&self, pixmap: &mut Pixmap) {
        use web_sys::WebGl2RenderingContext;

        let width = self.scene.width();
        let height = self.scene.height();

        let render_size = vello_hybrid::RenderSize {
            width: width.into(),
            height: height.into(),
        };

        let mut renderer = self.renderer.borrow_mut();
        renderer
            .render(&self.scene, &render_size)
            .expect("WebGL render failed");

        let gl = renderer.gl_context();

        let mut pixels = vec![0_u8; (width as usize) * (height as usize) * 4];
        gl.read_pixels_with_opt_u8_array(
            0,
            0,
            width.into(),
            height.into(),
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            Some(&mut pixels),
        )
        .unwrap();

        let pixmap_data = pixmap.data_as_u8_slice_mut();
        pixmap_data.copy_from_slice(&pixels);
    }

    fn width(&self) -> u16 {
        self.scene.width()
    }

    fn height(&self) -> u16 {
        self.scene.height()
    }

    fn get_image_source(&mut self, pixmap: Arc<Pixmap>) -> ImageSource {
        let image_id = self.renderer.borrow_mut().upload_image(&pixmap);
        ImageSource::OpaqueId(image_id)
    }

    fn record(&mut self, recording: &mut Recording, f: impl FnOnce(&mut Recorder<'_>)) {
        Recordable::record(&mut self.scene, recording, f);
    }

    fn prepare_recording(&mut self, recording: &mut Recording) {
        Recordable::prepare_recording(&mut self.scene, recording);
    }

    fn execute_recording(&mut self, recording: &Recording) {
        Recordable::execute_recording(&mut self.scene, recording);
    }
}
