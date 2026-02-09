//! Build script that auto-discovers `.anyrender.zip` scene files in the `scenes/` directory
//! and generates Rust source with `include_bytes!` for each file.
//!
//! Scene deserialization happens at runtime using `anyrender_serialize`.

use std::fs;
use std::path::Path;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let scenes_dir = Path::new(&manifest_dir).join("../scenes");
    let out_dir = std::env::var("OUT_DIR").unwrap();

    // Re-run if the scenes directory changes
    println!("cargo:rerun-if-changed=../scenes");

    let mut entries: Vec<(String, String)> = Vec::new();

    if scenes_dir.exists() && scenes_dir.is_dir() {
        let mut dir_entries: Vec<_> = fs::read_dir(&scenes_dir)
            .expect("Failed to read scenes directory")
            .filter_map(|e| e.ok())
            .collect();

        dir_entries.sort_by_key(|e| e.file_name());

        for entry in dir_entries {
            let path = entry.path();
            let file_name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            // Match files ending in .anyrender.zip
            if !file_name.ends_with(".anyrender.zip") {
                continue;
            }

            // Re-run if this individual scene file's content changes.
            println!("cargo:rerun-if-changed={}", path.display());

            // Derive scene name: "demo_scene.anyrender.zip" -> "demo_scene"
            let scene_name = file_name
                .strip_suffix(".anyrender.zip")
                .unwrap()
                .to_string();

            let abs_path = fs::canonicalize(&path)
                .unwrap_or_else(|e| panic!("Failed to canonicalize {}: {e}", path.display()));

            entries.push((scene_name, abs_path.display().to_string()));

            println!("cargo:warning=Found scene: {file_name}");
        }
    }

    // Generate scene_list.rs with raw ZIP bytes
    let mut code = String::from(
        "/// Auto-generated list of scene archive files.\n\
         /// Each entry is (scene_name, raw_zip_bytes).\n\
         pub static SCENE_FILES: &[(&str, &[u8])] = &[\n",
    );

    for (name, abs_path) in &entries {
        code.push_str(&format!(
            "    (\"{name}\", include_bytes!(\"{abs_path}\")),\n"
        ));
    }

    code.push_str("];\n");

    let scene_list_path = Path::new(&out_dir).join("scene_list.rs");
    fs::write(&scene_list_path, &code).unwrap();

    println!(
        "cargo:warning=Generated scene_list.rs with {} scene(s)",
        entries.len()
    );
}
