//! Programmatic scene definitions using the [`Renderer`] trait.
//!
//! Each scene is defined as a type implementing [`VelloScene`], which splits
//! into a **setup** phase (run once, for image uploads etc.) and a **draw**
//! phase (run in the benchmark hot loop).
//!
//! Scene files are organised by category:
//! - [`filled_rects`] — simple vector-only scenes.
//! - [`images`] — image-heavy scenes at varying counts.
//!
//! To add a new category, create a new sub-module and register its scenes
//! in the [`register_vello_scenes!`] invocation at the bottom of this file.

mod filled_rects;
pub mod images;

use crate::renderer::Renderer;

// Re-export scene types so external code can reference them if needed.
pub use filled_rects::FilledRects;
pub use images::*;

/// Metadata for a programmatic vello scene.
#[derive(Debug, Clone)]
pub struct VelloSceneInfo {
    pub name: &'static str,
    pub width: u16,
    pub height: u16,
}

/// A scene defined via the [`Renderer`] trait.
///
/// - [`setup`](VelloScene::setup) runs once before measurement and may perform
///   expensive operations such as image uploads via
///   [`Renderer::get_image_source`].
/// - [`draw`](VelloScene::draw) runs in the benchmark hot loop using the
///   pre-computed state from `setup`.
pub trait VelloScene {
    /// Opaque state produced by `setup` (e.g. uploaded [`ImageSource`] handles).
    /// Use `()` when no setup state is needed.
    type State: 'static;

    /// Scene metadata (name, dimensions).
    fn info() -> VelloSceneInfo;

    /// One-time setup. Image uploads and other expensive work happen here.
    fn setup<R: Renderer>(r: &mut R) -> Self::State;

    /// Draw the scene. Called in the benchmark hot loop.
    fn draw<R: Renderer>(state: &Self::State, r: &mut R);
}

// ===========================================================================
// Registration macro & dispatch
// ===========================================================================

/// Register all vello scenes. Add new scene types to the list below.
///
/// This macro generates generic dispatch functions that work with any
/// backend implementing [`Renderer`]:
/// - `get_vello_scenes()` — list of all scene metadata
/// - `setup_scene<R>()` — run setup for a scene by name
/// - `draw_scene<R>()` — draw a scene by name with pre-computed state
macro_rules! register_vello_scenes {
    ($(($name_str:expr, $scene:ty)),* $(,)?) => {
        /// Get metadata for all registered vello scenes.
        pub fn get_vello_scenes() -> Vec<VelloSceneInfo> {
            vec![$(<$scene as VelloScene>::info()),*]
        }

        /// Run setup for a scene by name using any [`Renderer`] backend.
        /// Returns a boxed state that must be passed to [`draw_scene`].
        pub fn setup_scene<R: Renderer>(
            name: &str,
            r: &mut R,
        ) -> Option<Box<dyn std::any::Any>> {
            match name {
                $($name_str => {
                    let state = <$scene as VelloScene>::setup(r);
                    Some(Box::new(state))
                }),*
                _ => None,
            }
        }

        /// Draw a scene by name using any [`Renderer`] backend with
        /// pre-computed state from [`setup_scene`].
        pub fn draw_scene<R: Renderer>(
            name: &str,
            state: &dyn std::any::Any,
            r: &mut R,
        ) {
            match name {
                $($name_str => {
                    let state = state
                        .downcast_ref::<<$scene as VelloScene>::State>()
                        .expect("state type mismatch");
                    <$scene as VelloScene>::draw(state, r);
                }),*
                _ => panic!("unknown vello scene: {name}"),
            }
        }
    };
}

// Register all scenes here.
register_vello_scenes!(
    // Vector-only
    ("filled_rects", FilledRects),
    // Tiled flowers
    ("tiled_flowers_100", TiledFlowers100),
    ("tiled_flowers_300", TiledFlowers300),
    ("tiled_flowers_1000", TiledFlowers1000),
    ("tiled_flowers_10000", TiledFlowers10000),
    // Overlapping images (opaque)
    ("overlapping_images_100", OverlappingImages100),
    ("overlapping_images_1000", OverlappingImages1000),
    ("overlapping_images_10000", OverlappingImages10000),
    // Clipped image cards
    ("clipped_image_cards_100", ClippedImageCards100),
    ("clipped_image_cards_1000", ClippedImageCards1000),
    ("clipped_image_cards_10000", ClippedImageCards10000),
    // Large overlapping images (opaque, no alpha)
    ("large_overlapping_images_100", LargeOverlappingImages100),
    ("large_overlapping_images_1000", LargeOverlappingImages1000),
    ("large_overlapping_images_10000", LargeOverlappingImages10000),
    // Rotated images
    ("rotated_images_100", RotatedImages100),
    ("rotated_images_1000", RotatedImages1000),
    ("rotated_images_10000", RotatedImages10000),
    // Image cards with SVG-style borders
    ("image_cards_with_borders_100", ImageCardsWithBorders100),
    ("image_cards_with_borders_1000", ImageCardsWithBorders1000),
    ("image_cards_with_borders_10000", ImageCardsWithBorders10000),
    // Mixed image and vector
    ("mixed_image_and_vector_100", MixedImageAndVector100),
    ("mixed_image_and_vector_1000", MixedImageAndVector1000),
    ("mixed_image_and_vector_10000", MixedImageAndVector10000),
    // Paths and images — 100 random SVG paths then 1 image, repeated
    ("paths_and_images_100", PathsAndImages100),
);
