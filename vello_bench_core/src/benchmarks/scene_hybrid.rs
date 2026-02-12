//! Benchmarks that replay serialized AnyRender scenes using Vello Hybrid.
//!
//! On native (non-WASM): uses wgpu for headless GPU rendering.
//! On WASM: hybrid benchmarks are handled by the `vello_bench_wasm` crate
//! on the main thread using WebGL (not available in this core crate).
//!
//! Each scene in the `scenes/` directory becomes a benchmark under the
//! `scene_hybrid` category. The benchmark measures the full hybrid
//! rendering pipeline: scene replay + GPU rendering + GPU sync.

use crate::registry::BenchmarkInfo;
use crate::result::BenchmarkResult;
use crate::runner::BenchRunner;
use crate::scenes::get_scenes;
use fearless_simd::Level;

const CATEGORY: &str = "scene_hybrid";

/// Encapsulates all state needed to render a scene with the Vello Hybrid
/// (wgpu) backend.
///
/// Used by both benchmarks (hot loop) and screenshots (single render) to
/// ensure the exact same codepath. Native-only — WASM hybrid rendering is
/// handled by `vello_bench_wasm`.
#[cfg(not(target_arch = "wasm32"))]
pub struct HybridSceneRenderer {
    gpu: GpuContext,
    renderer: vello_hybrid::Renderer,
    hybrid_scene: vello_hybrid::Scene,
    render_size: vello_hybrid::RenderSize,
    ctx: anyrender_vello_hybrid::VelloHybridRenderContext,
    scene: anyrender::Scene,
}

#[cfg(not(target_arch = "wasm32"))]
impl HybridSceneRenderer {
    /// Set up a Hybrid renderer for the given scene (initialises wgpu).
    pub fn new(item: &crate::scenes::SceneItem) -> Self {
        let width = item.width as u32;
        let height = item.height as u32;

        let gpu = pollster::block_on(init_gpu(width, height));

        let render_target_config = vello_hybrid::RenderTargetConfig {
            format: wgpu::TextureFormat::Rgba8Unorm,
            width,
            height,
        };

        let renderer = vello_hybrid::Renderer::new(&gpu.device, &render_target_config);
        let hybrid_scene = vello_hybrid::Scene::new(item.width, item.height);
        let render_size = vello_hybrid::RenderSize { width, height };

        let mut ctx = anyrender_vello_hybrid::VelloHybridRenderContext::new();
        let scene = item
            .archive
            .to_scene(&mut ctx)
            .expect("Failed to deserialize scene for Hybrid backend");

        Self {
            gpu,
            renderer,
            hybrid_scene,
            render_size,
            ctx,
            scene,
        }
    }

    /// Render one frame. This is the benchmarked operation.
    #[inline(always)]
    pub fn render_frame(&mut self) {
        use anyrender::PaintScene;
        use anyrender_vello_hybrid::VelloHybridScenePainter;
        use vello_common::kurbo::Affine;

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let texture_view = self
            .gpu
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Build the scene
        {
            let mut painter = VelloHybridScenePainter::new(
                &mut self.ctx,
                &mut self.renderer,
                &self.gpu.device,
                &self.gpu.queue,
                &mut self.hybrid_scene,
            );
            painter.append_scene(self.scene.clone(), Affine::IDENTITY);
        }

        self.renderer
            .render(
                &self.hybrid_scene,
                &self.gpu.device,
                &self.gpu.queue,
                &mut encoder,
                &self.render_size,
                &texture_view,
            )
            .expect("Hybrid render failed");

        self.gpu.queue.submit(Some(encoder.finish()));
        self.gpu
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();

        self.hybrid_scene.reset();
    }

    /// Consume the renderer, do one final render, and read the GPU texture
    /// back to a CPU buffer as non-premultiplied RGBA8.
    pub fn into_rgba(mut self) -> Vec<u8> {
        // Ensure there is a rendered frame on the texture.
        self.render_frame();

        let width = self.render_size.width;
        let height = self.render_size.height;

        let bytes_per_row = align_to(width * 4, 256);
        let readback_buffer = self.gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("screenshot_readback"),
            size: (bytes_per_row * height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.gpu.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        self.gpu.queue.submit(Some(encoder.finish()));
        self.gpu
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();

        let buffer_slice = readback_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        self.gpu
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
        rx.recv().unwrap().expect("Failed to map buffer");

        let data = buffer_slice.get_mapped_range();

        // Strip row padding (bytes_per_row may be larger than width * 4).
        let row_bytes = (width * 4) as usize;
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for row in 0..height as usize {
            let start = row * bytes_per_row as usize;
            rgba.extend_from_slice(&data[start..start + row_bytes]);
        }

        drop(data);
        readback_buffer.unmap();

        // Rgba8Unorm is already non-premultiplied — no conversion needed.
        rgba
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
        // Hybrid WASM benchmarks are handled by vello_bench_wasm on the main thread.
        None
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn run_native(name: &str, runner: &BenchRunner, level: Level) -> Option<BenchmarkResult> {
    use crate::simd::level_suffix;

    let scenes = get_scenes();
    let item = scenes.iter().find(|s| s.name == name)?;
    let simd_variant = level_suffix(level);

    let mut renderer = HybridSceneRenderer::new(item);

    Some(runner.run(
        &format!("{CATEGORY}/{name}"),
        CATEGORY,
        name,
        simd_variant,
        #[inline(always)]
        || {
            renderer.render_frame();
        },
    ))
}

#[cfg(not(target_arch = "wasm32"))]
struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    texture: wgpu::Texture,
}

#[cfg(not(target_arch = "wasm32"))]
async fn init_gpu(width: u32, height: u32) -> GpuContext {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        })
        .await
        .expect("Failed to find a suitable GPU adapter");

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .expect("Failed to create GPU device");

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("bench_render_target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    GpuContext {
        device,
        queue,
        texture,
    }
}

/// Round `value` up to the next multiple of `alignment`.
#[cfg(not(target_arch = "wasm32"))]
fn align_to(value: u32, alignment: u32) -> u32 {
    (value + alignment - 1) / alignment * alignment
}
