//! Image-heavy benchmark scenes.
//!
//! All scenes in this module share a single uploaded image (`splash-flower.jpg`)
//! via [`ImageGridState`]. The image is uploaded once during [`VelloScene::setup`]
//! and referenced by opaque handle in the draw loop.
//!
//! To add a new image scene:
//! 1. Write a `fn draw_my_scene<R: Renderer>(state: &ImageGridState, r: &mut R, count: u32)`.
//! 2. Stamp out variants with the [`counted_image_scene!`] macro.
//! 3. Register them in `mod.rs`'s `register_vello_scenes!` invocation.

use std::sync::Arc;

use super::{VelloScene, VelloSceneInfo};
use crate::renderer::Renderer;
use vello_common::kurbo::{Affine, BezPath, Rect, RoundedRect, Shape, Stroke};
use vello_common::paint::{Image, ImageSource};
use vello_common::peniko::color::palette;
use vello_common::peniko::color::PremulRgba8;
use vello_common::peniko::ImageSampler;
use vello_common::pixmap::Pixmap;

// ===========================================================================
// Shared helpers
// ===========================================================================

/// Decode the embedded splash-flower JPEG into a premultiplied-alpha [`Pixmap`].
fn load_splash_flower_pixmap() -> Pixmap {
    static JPEG_BYTES: &[u8] = include_bytes!("../../assets/splash-flower.jpg");

    let img = image::load_from_memory_with_format(JPEG_BYTES, image::ImageFormat::Jpeg)
        .expect("failed to decode splash-flower.jpg")
        .into_rgba8();

    let (w, h) = img.dimensions();

    #[expect(
        clippy::cast_possible_truncation,
        reason = "Image is known to be small enough."
    )]
    let pixels: Vec<PremulRgba8> = img
        .pixels()
        .map(|p| PremulRgba8 {
            r: p[0],
            g: p[1],
            b: p[2],
            a: p[3],
        })
        .collect();

    Pixmap::from_parts(pixels, w as u16, h as u16)
}

/// Shared state for image scenes: an uploaded image handle + dimensions.
pub struct ImageGridState {
    image_source: ImageSource,
    img_w: u16,
    img_h: u16,
}

pub(super) fn setup_image_grid<R: Renderer>(r: &mut R) -> ImageGridState {
    let pixmap = load_splash_flower_pixmap();
    let img_w = pixmap.width();
    let img_h = pixmap.height();
    let image_source = r.get_image_source(Arc::new(pixmap));
    ImageGridState {
        image_source,
        img_w,
        img_h,
    }
}

// ===========================================================================
// Parameterized draw functions
// ===========================================================================

/// Draw `count` images in a non-overlapping grid.
fn draw_tiled_flowers<R: Renderer>(state: &ImageGridState, r: &mut R, count: u32) {
    let canvas_w = f64::from(r.width());
    let canvas_h = f64::from(r.height());
    let img_w = f64::from(state.img_w);
    let img_h = f64::from(state.img_h);

    let aspect = canvas_w / canvas_h;
    let rows_f = (f64::from(count) / aspect).sqrt();
    let cols = (rows_f * aspect).ceil() as u32;
    let rows = rows_f.ceil() as u32;

    let cell_w = canvas_w / f64::from(cols);
    let cell_h = canvas_h / f64::from(rows);
    let sx = cell_w / img_w;
    let sy = cell_h / img_h;

    let mut n = 0u32;
    for row in 0..rows {
        for col in 0..cols {
            if n >= count {
                r.set_transform(Affine::IDENTITY);
                return;
            }
            n += 1;

            let x = f64::from(col) * cell_w;
            let y = f64::from(row) * cell_h;

            r.set_transform(Affine::translate((x, y)) * Affine::scale_non_uniform(sx, sy));
            r.set_paint(Image {
                image: state.image_source.clone(),
                sampler: ImageSampler::default(),
            });
            r.fill_rect(&Rect::new(0.0, 0.0, img_w, img_h));
        }
    }
    r.set_transform(Affine::IDENTITY);
}

/// Draw `count` overlapping opaque images at pseudo-random positions.
fn draw_overlapping_images<R: Renderer>(state: &ImageGridState, r: &mut R, count: u32) {
    let canvas_w = f64::from(r.width());
    let canvas_h = f64::from(r.height());
    let img_w = f64::from(state.img_w);
    let img_h = f64::from(state.img_h);

    let tile_w = canvas_w / 12.0;
    let tile_h = canvas_h / 8.0;
    let sx = tile_w / img_w;
    let sy = tile_h / img_h;

    for i in 0..count {
        let fx = (i as f64 * 97.0) % canvas_w;
        let fy = (i as f64 * 53.0) % canvas_h;

        r.set_transform(Affine::translate((fx, fy)) * Affine::scale_non_uniform(sx, sy));
        r.set_paint(Image {
            image: state.image_source.clone(),
            sampler: ImageSampler::default(),
        });
        r.fill_rect(&Rect::new(0.0, 0.0, img_w, img_h));
    }
    r.set_transform(Affine::IDENTITY);
}

/// Draw `count` images each clipped to a rounded rectangle with a stroked border.
fn draw_clipped_image_cards<R: Renderer>(state: &ImageGridState, r: &mut R, count: u32) {
    let canvas_w = f64::from(r.width());
    let canvas_h = f64::from(r.height());
    let img_w = f64::from(state.img_w);
    let img_h = f64::from(state.img_h);

    let cols = ((count as f64).sqrt() * (canvas_w / canvas_h).sqrt()).ceil() as u32;
    let rows = (count + cols - 1) / cols;
    let padding = 4.0;
    let cell_w = canvas_w / f64::from(cols);
    let cell_h = canvas_h / f64::from(rows);
    let card_w = cell_w - padding * 2.0;
    let card_h = cell_h - padding * 2.0;
    let corner_radius = 8.0;
    let sx = card_w / img_w;
    let sy = card_h / img_h;

    let border_stroke = Stroke {
        width: 2.0,
        ..Default::default()
    };

    let mut n = 0u32;
    for row in 0..rows {
        for col in 0..cols {
            if n >= count {
                return;
            }
            n += 1;

            let x = f64::from(col) * cell_w + padding;
            let y = f64::from(row) * cell_h + padding;

            let rrect = RoundedRect::new(x, y, x + card_w, y + card_h, corner_radius);
            let clip_path = rrect.to_path(0.1);

            r.push_clip_layer(&clip_path);
            r.set_transform(Affine::translate((x, y)) * Affine::scale_non_uniform(sx, sy));
            r.set_paint(Image {
                image: state.image_source.clone(),
                sampler: ImageSampler::default(),
            });
            r.fill_rect(&Rect::new(0.0, 0.0, img_w, img_h));
            r.set_transform(Affine::IDENTITY);
            r.pop_layer();

            r.set_stroke(border_stroke.clone());
            r.set_paint(palette::css::WHITE);
            r.stroke_path(&clip_path);
        }
    }
}

/// Draw `count` large overlapping opaque images (no alpha) sweeping diagonally.
fn draw_large_overlapping_images<R: Renderer>(state: &ImageGridState, r: &mut R, count: u32) {
    let canvas_w = f64::from(r.width());
    let canvas_h = f64::from(r.height());
    let img_w = f64::from(state.img_w);
    let img_h = f64::from(state.img_h);

    let draw_w = canvas_w * 0.4;
    let draw_h = canvas_h * 0.4;
    let sx = draw_w / img_w;
    let sy = draw_h / img_h;

    for i in 0..count {
        let t = i as f64 / f64::from(count);
        let x = t * (canvas_w - draw_w);
        let y = t * (canvas_h - draw_h);

        r.set_transform(Affine::translate((x, y)) * Affine::scale_non_uniform(sx, sy));
        r.set_paint(Image {
            image: state.image_source.clone(),
            sampler: ImageSampler::default(),
        });
        r.fill_rect(&Rect::new(0.0, 0.0, img_w, img_h));
    }
    r.set_transform(Affine::IDENTITY);
}

/// Draw `count` images each rotated by a different angle.
fn draw_rotated_images<R: Renderer>(state: &ImageGridState, r: &mut R, count: u32) {
    let canvas_w = f64::from(r.width());
    let canvas_h = f64::from(r.height());
    let img_w = f64::from(state.img_w);
    let img_h = f64::from(state.img_h);

    let cols = ((count as f64).sqrt() * (canvas_w / canvas_h).sqrt()).ceil() as u32;
    let rows = (count + cols - 1) / cols;
    let cell_w = canvas_w / f64::from(cols);
    let cell_h = canvas_h / f64::from(rows);
    let tile = cell_w.min(cell_h) * 0.6;
    let sx = tile / img_w;
    let sy = tile / img_h;

    let mut n = 0u32;
    for row in 0..rows {
        for col in 0..cols {
            if n >= count {
                r.set_transform(Affine::IDENTITY);
                return;
            }
            let angle = (n as f64) * std::f64::consts::TAU / f64::from(count);
            n += 1;

            let cx = f64::from(col) * cell_w + cell_w * 0.5;
            let cy = f64::from(row) * cell_h + cell_h * 0.5;

            r.set_transform(
                Affine::translate((cx, cy))
                    * Affine::rotate(angle)
                    * Affine::scale_non_uniform(sx, sy)
                    * Affine::translate((-img_w * 0.5, -img_h * 0.5)),
            );
            r.set_paint(Image {
                image: state.image_source.clone(),
                sampler: ImageSampler::default(),
            });
            r.fill_rect(&Rect::new(0.0, 0.0, img_w, img_h));
        }
    }
    r.set_transform(Affine::IDENTITY);
}

/// Draw `count` images with decorative SVG-style double borders.
fn draw_image_cards_with_borders<R: Renderer>(state: &ImageGridState, r: &mut R, count: u32) {
    let canvas_w = f64::from(r.width());
    let canvas_h = f64::from(r.height());
    let img_w = f64::from(state.img_w);
    let img_h = f64::from(state.img_h);

    let cols = ((count as f64).sqrt() * (canvas_w / canvas_h).sqrt()).ceil() as u32;
    let rows = (count + cols - 1) / cols;
    let padding = 6.0;
    let cell_w = canvas_w / f64::from(cols);
    let cell_h = canvas_h / f64::from(rows);
    let card_w = cell_w - padding * 2.0;
    let card_h = cell_h - padding * 2.0;
    let corner = 10.0;
    let sx = card_w / img_w;
    let sy = card_h / img_h;

    let thin_stroke = Stroke {
        width: 1.5,
        ..Default::default()
    };
    let thick_stroke = Stroke {
        width: 3.0,
        ..Default::default()
    };
    let colors = [
        palette::css::CORNFLOWER_BLUE,
        palette::css::CORAL,
        palette::css::MEDIUM_SEA_GREEN,
        palette::css::GOLD,
        palette::css::ORCHID,
        palette::css::TOMATO,
    ];

    let mut n = 0u32;
    for row in 0..rows {
        for col in 0..cols {
            if n >= count {
                return;
            }
            let color = colors[n as usize % colors.len()];
            n += 1;

            let x = f64::from(col) * cell_w + padding;
            let y = f64::from(row) * cell_h + padding;

            // Outer decorative border.
            let outer = RoundedRect::new(
                x - 2.0,
                y - 2.0,
                x + card_w + 2.0,
                y + card_h + 2.0,
                corner + 2.0,
            );
            let outer_path = outer.to_path(0.1);
            r.set_stroke(thick_stroke.clone());
            r.set_paint(color);
            r.stroke_path(&outer_path);

            // Inner card — clip + image.
            let inner = RoundedRect::new(x, y, x + card_w, y + card_h, corner);
            let inner_path = inner.to_path(0.1);

            r.push_clip_layer(&inner_path);
            r.set_transform(Affine::translate((x, y)) * Affine::scale_non_uniform(sx, sy));
            r.set_paint(Image {
                image: state.image_source.clone(),
                sampler: ImageSampler::default(),
            });
            r.fill_rect(&Rect::new(0.0, 0.0, img_w, img_h));
            r.set_transform(Affine::IDENTITY);
            r.pop_layer();

            // Inner thin white highlight.
            r.set_stroke(thin_stroke.clone());
            r.set_paint(palette::css::WHITE);
            r.stroke_path(&inner_path);
        }
    }
}

/// Draw `count` elements alternating between image tiles and vector rects.
fn draw_mixed_image_and_vector<R: Renderer>(state: &ImageGridState, r: &mut R, count: u32) {
    let canvas_w = f64::from(r.width());
    let canvas_h = f64::from(r.height());
    let img_w = f64::from(state.img_w);
    let img_h = f64::from(state.img_h);

    let cols = ((count as f64).sqrt() * (canvas_w / canvas_h).sqrt()).ceil() as u32;
    let rows = (count + cols - 1) / cols;
    let cell_w = canvas_w / f64::from(cols);
    let cell_h = canvas_h / f64::from(rows);
    let sx = cell_w / img_w;
    let sy = cell_h / img_h;

    let colors = [
        palette::css::STEEL_BLUE,
        palette::css::INDIAN_RED,
        palette::css::DARK_SEA_GREEN,
        palette::css::DARK_ORANGE,
    ];
    let border_stroke = Stroke {
        width: 1.0,
        ..Default::default()
    };

    let mut n = 0u32;
    for row in 0..rows {
        for col in 0..cols {
            if n >= count {
                return;
            }
            let x = f64::from(col) * cell_w;
            let y = f64::from(row) * cell_h;

            if n % 2 == 0 {
                r.set_transform(Affine::translate((x, y)) * Affine::scale_non_uniform(sx, sy));
                r.set_paint(Image {
                    image: state.image_source.clone(),
                    sampler: ImageSampler::default(),
                });
                r.fill_rect(&Rect::new(0.0, 0.0, img_w, img_h));
                r.set_transform(Affine::IDENTITY);
            } else {
                let rect = Rect::new(x + 1.0, y + 1.0, x + cell_w - 1.0, y + cell_h - 1.0);
                r.set_paint(colors[n as usize % colors.len()]);
                r.fill_rect(&rect);
                r.set_stroke(border_stroke.clone());
                r.set_paint(palette::css::WHITE);
                r.stroke_rect(&rect);
            }
            n += 1;
        }
    }
}

/// Draw a scene that interleaves batches of random SVG paths with images.
///
/// For each of `iterations` rounds: draw `paths_per_batch` filled bezier
/// paths (deterministic pseudo-random curves) and then one image.
/// Total elements = iterations * (paths_per_batch + 1).
fn draw_paths_and_images<R: Renderer>(
    state: &ImageGridState,
    r: &mut R,
    iterations: u32,
    paths_per_batch: u32,
) {
    let canvas_w = f64::from(r.width());
    let canvas_h = f64::from(r.height());
    let img_w = f64::from(state.img_w);
    let img_h = f64::from(state.img_h);

    let img_cols = (iterations as f64).sqrt().ceil() as u32;
    let img_rows = (iterations + img_cols - 1) / img_cols;
    let cell_w = canvas_w / f64::from(img_cols);
    let cell_h = canvas_h / f64::from(img_rows);
    let sx = cell_w / img_w;
    let sy = cell_h / img_h;

    let path_colors = [
        palette::css::RED,
        palette::css::LIME,
        palette::css::BLUE,
        palette::css::ORANGE,
        palette::css::PURPLE,
        palette::css::TEAL,
        palette::css::CRIMSON,
        palette::css::DARK_CYAN,
    ];

    // Simple deterministic LCG for reproducible "random" coordinates.
    let mut seed: u64 = 12345;
    let mut rng = || -> f64 {
        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        (seed >> 33) as f64 / (1u64 << 31) as f64
    };

    let path_stroke = Stroke {
        width: 1.5,
        ..Default::default()
    };

    for iter in 0..iterations {
        for p in 0..paths_per_batch {
            let global_idx = (iter * paths_per_batch + p) as usize;
            let color = path_colors[global_idx % path_colors.len()];

            let mut path = BezPath::new();
            let x0 = rng() * canvas_w;
            let y0 = rng() * canvas_h;
            path.move_to((x0, y0));

            let seg_count = 4 + (global_idx % 3);
            for _ in 0..seg_count {
                match global_idx % 3 {
                    0 => {
                        path.line_to((rng() * canvas_w, rng() * canvas_h));
                    }
                    1 => {
                        path.quad_to(
                            (rng() * canvas_w, rng() * canvas_h),
                            (rng() * canvas_w, rng() * canvas_h),
                        );
                    }
                    _ => {
                        path.curve_to(
                            (rng() * canvas_w, rng() * canvas_h),
                            (rng() * canvas_w, rng() * canvas_h),
                            (rng() * canvas_w, rng() * canvas_h),
                        );
                    }
                }
            }
            path.close_path();

            if global_idx % 2 == 0 {
                r.set_paint(color);
                r.fill_path(&path);
            } else {
                r.set_stroke(path_stroke.clone());
                r.set_paint(color);
                r.stroke_path(&path);
            }
        }

        let col = iter % img_cols;
        let row = iter / img_cols;
        let x = f64::from(col) * cell_w;
        let y = f64::from(row) * cell_h;

        r.set_transform(Affine::translate((x, y)) * Affine::scale_non_uniform(sx, sy));
        r.set_paint(Image {
            image: state.image_source.clone(),
            sampler: ImageSampler::default(),
        });
        r.fill_rect(&Rect::new(0.0, 0.0, img_w, img_h));
        r.set_transform(Affine::IDENTITY);
    }
}

// ===========================================================================
// Macro to stamp out VelloScene impls at specific counts
// ===========================================================================

/// Generate a scene struct + [`VelloScene`] impl that delegates to a
/// parameterized draw function with a fixed count.
macro_rules! counted_image_scene {
    (
        struct $name:ident,
        bench_name: $bench_name:expr,
        count: $count:expr,
        draw_fn: $draw_fn:ident $(,)?
    ) => {
        pub struct $name;

        impl VelloScene for $name {
            type State = ImageGridState;

            fn info() -> VelloSceneInfo {
                VelloSceneInfo {
                    name: $bench_name,
                    width: 1920,
                    height: 1080,
                }
            }

            fn setup<R: Renderer>(r: &mut R) -> Self::State {
                setup_image_grid(r)
            }

            fn draw<R: Renderer>(state: &Self::State, r: &mut R) {
                $draw_fn(state, r, $count);
            }
        }
    };
}

// Tiled flowers — non-overlapping grid
counted_image_scene!(struct TiledFlowers100,   bench_name: "tiled_flowers_100",   count: 100,   draw_fn: draw_tiled_flowers);
counted_image_scene!(struct TiledFlowers300,   bench_name: "tiled_flowers_300",   count: 300,   draw_fn: draw_tiled_flowers);
counted_image_scene!(struct TiledFlowers1000,  bench_name: "tiled_flowers_1000",  count: 1000,  draw_fn: draw_tiled_flowers);
counted_image_scene!(struct TiledFlowers10000, bench_name: "tiled_flowers_10000", count: 10000, draw_fn: draw_tiled_flowers);

// Overlapping images — opaque, pseudo-random positions
counted_image_scene!(struct OverlappingImages100,   bench_name: "overlapping_images_100",   count: 100,   draw_fn: draw_overlapping_images);
counted_image_scene!(struct OverlappingImages1000,  bench_name: "overlapping_images_1000",  count: 1000,  draw_fn: draw_overlapping_images);
counted_image_scene!(struct OverlappingImages10000, bench_name: "overlapping_images_10000", count: 10000, draw_fn: draw_overlapping_images);

// Clipped image cards — rounded-rect clip + stroked border
counted_image_scene!(struct ClippedImageCards100,   bench_name: "clipped_image_cards_100",   count: 100,   draw_fn: draw_clipped_image_cards);
counted_image_scene!(struct ClippedImageCards1000,  bench_name: "clipped_image_cards_1000",  count: 1000,  draw_fn: draw_clipped_image_cards);
counted_image_scene!(struct ClippedImageCards10000, bench_name: "clipped_image_cards_10000", count: 10000, draw_fn: draw_clipped_image_cards);

// Large overlapping images — opaque, heavy overdraw
counted_image_scene!(struct LargeOverlappingImages100,   bench_name: "large_overlapping_images_100",   count: 100,   draw_fn: draw_large_overlapping_images);
counted_image_scene!(struct LargeOverlappingImages1000,  bench_name: "large_overlapping_images_1000",  count: 1000,  draw_fn: draw_large_overlapping_images);
counted_image_scene!(struct LargeOverlappingImages10000, bench_name: "large_overlapping_images_10000", count: 10000, draw_fn: draw_large_overlapping_images);

// Rotated images — non-axis-aligned sampling
counted_image_scene!(struct RotatedImages100,   bench_name: "rotated_images_100",   count: 100,   draw_fn: draw_rotated_images);
counted_image_scene!(struct RotatedImages1000,  bench_name: "rotated_images_1000",  count: 1000,  draw_fn: draw_rotated_images);
counted_image_scene!(struct RotatedImages10000, bench_name: "rotated_images_10000", count: 10000, draw_fn: draw_rotated_images);

// Image cards with SVG-style borders — clip + double stroke
counted_image_scene!(struct ImageCardsWithBorders100,   bench_name: "image_cards_with_borders_100",   count: 100,   draw_fn: draw_image_cards_with_borders);
counted_image_scene!(struct ImageCardsWithBorders1000,  bench_name: "image_cards_with_borders_1000",  count: 1000,  draw_fn: draw_image_cards_with_borders);
counted_image_scene!(struct ImageCardsWithBorders10000, bench_name: "image_cards_with_borders_10000", count: 10000, draw_fn: draw_image_cards_with_borders);

// Mixed image and vector — alternating image tiles and coloured rects
counted_image_scene!(struct MixedImageAndVector100,   bench_name: "mixed_image_and_vector_100",   count: 100,   draw_fn: draw_mixed_image_and_vector);
counted_image_scene!(struct MixedImageAndVector1000,  bench_name: "mixed_image_and_vector_1000",  count: 1000,  draw_fn: draw_mixed_image_and_vector);
counted_image_scene!(struct MixedImageAndVector10000, bench_name: "mixed_image_and_vector_10000", count: 10000, draw_fn: draw_mixed_image_and_vector);

// Paths and images — 100 random SVG paths then 1 image, repeated 100 times
/// 100 iterations of (100 random SVG paths + 1 image) = 10,000 paths + 100 images.
pub struct PathsAndImages100;

impl VelloScene for PathsAndImages100 {
    type State = ImageGridState;

    fn info() -> VelloSceneInfo {
        VelloSceneInfo {
            name: "paths_and_images_100",
            width: 1920,
            height: 1080,
        }
    }

    fn setup<R: Renderer>(r: &mut R) -> Self::State {
        setup_image_grid(r)
    }

    fn draw<R: Renderer>(state: &Self::State, r: &mut R) {
        draw_paths_and_images(state, r, 10, 100);
    }
}
