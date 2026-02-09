//! Scene loading from AnyRender scene archives.
//!
//! Scene files are auto-discovered from the `scenes/` directory at build time
//! by the build script, which generates `include_bytes!` entries for each
//! `.anyrender.zip` file. At runtime, scenes are lazily deserialized from the
//! embedded ZIP data using `anyrender_serialize`.

use std::io::Cursor;
use std::sync::OnceLock;

// Include the auto-generated scene list from the build script.
include!(concat!(env!("OUT_DIR"), "/scene_list.rs"));

/// Default render width for scenes without explicit dimensions.
pub const DEFAULT_SCENE_WIDTH: u16 = 1024;
/// Default render height for scenes without explicit dimensions.
pub const DEFAULT_SCENE_HEIGHT: u16 = 768;

/// A loaded scene ready for benchmarking.
pub struct SceneItem {
    /// Human-readable name derived from the file name.
    pub name: String,
    /// The parsed scene archive.
    pub archive: anyrender_serialize::SceneArchive,
    /// Render width.
    pub width: u16,
    /// Render height.
    pub height: u16,
}

static SCENES: OnceLock<Vec<SceneItem>> = OnceLock::new();

/// Get the list of all loaded scenes (lazily deserialized on first access).
pub fn get_scenes() -> &'static [SceneItem] {
    SCENES.get_or_init(|| {
        SCENE_FILES
            .iter()
            .filter_map(|(name, zip_bytes)| {
                match load_archive_from_zip(zip_bytes) {
                    Ok(archive) => Some(SceneItem {
                        name: (*name).to_string(),
                        archive,
                        width: DEFAULT_SCENE_WIDTH,
                        height: DEFAULT_SCENE_HEIGHT,
                    }),
                    Err(e) => {
                        // Log but don't panic — allow other scenes to load.
                        #[cfg(target_arch = "wasm32")]
                        web_sys::console::error_1(
                            &format!("Failed to load scene '{name}': {e}").into(),
                        );
                        #[cfg(not(target_arch = "wasm32"))]
                        eprintln!("Failed to load scene '{name}': {e}");
                        None
                    }
                }
            })
            .collect()
    })
}

/// Parse a scene archive from ZIP bytes.
fn load_archive_from_zip(
    zip_bytes: &[u8],
) -> Result<anyrender_serialize::SceneArchive, Box<dyn std::error::Error>> {
    let cursor = Cursor::new(zip_bytes);
    Ok(anyrender_serialize::SceneArchive::deserialize(cursor)?)
}
