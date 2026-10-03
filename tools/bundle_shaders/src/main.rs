//! Lays out the `processing` and `lygia` WESL packages under `mewnala/shaders` so
//! the wheel ships them for editor tooling (`python -m mewnala shaders init`).

use std::fs;
use std::io;
use std::path::Path;

// wgsl-analyzer rejects lygia's own `unstable_2025` edition.
const PACKAGE_MANIFEST: &str = "edition = \"2026_pre\"\nroot = \".\"\n";

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
}

fn copy_wesl_tree(src: &Path, dst: &Path) -> io::Result<usize> {
    let mut count = 0;
    let mut entries = fs::read_dir(src)?.collect::<io::Result<Vec<_>>>()?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name();
        if entry.file_type()?.is_dir() {
            if name == "test" || name == "node_modules" || name.to_string_lossy().starts_with('.') {
                continue;
            }
            count += copy_wesl_tree(&path, &dst.join(&name))?;
        } else if path.extension().is_some_and(|e| e == "wesl") {
            fs::create_dir_all(dst)?;
            fs::copy(&path, dst.join(&name))?;
            count += 1;
        }
    }
    Ok(count)
}

fn bundle_processing(root: &Path, out: &Path) -> io::Result<usize> {
    let shaders = root.join("crates/processing_render/shaders");
    let dst = out.join("processing");
    // the wesl 0.3 `PkgBuilder` takes the root module from the sibling
    // `processing.wesl`, the spec and wgsl-analyzer expect `package.wesl`
    let count = copy_wesl_tree(&shaders.join("processing"), &dst)?;
    fs::copy(shaders.join("processing.wesl"), dst.join("package.wesl"))?;
    fs::write(dst.join("wesl.toml"), PACKAGE_MANIFEST)?;
    Ok(count + 1)
}

fn bundle_lygia(root: &Path, out: &Path) -> io::Result<usize> {
    let src = root.join("lygia");
    if !src.join("wesl.toml").is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "lygia submodule is not checked out (git submodule update --init)",
        ));
    }
    let dst = out.join("lygia");
    // `.wesl` only: the analyzer ignores `include` and would index the sibling
    // `.wgsl` copies, most of which still use `#include`
    let count = copy_wesl_tree(&src, &dst)?;
    fs::copy(src.join("LICENSE.md"), dst.join("LICENSE.md"))?;
    fs::write(dst.join("wesl.toml"), PACKAGE_MANIFEST)?;
    Ok(count)
}

fn main() -> io::Result<()> {
    let root = workspace_root();
    let out = root.join("crates/processing_pyo3/mewnala/shaders");
    if out.exists() {
        fs::remove_dir_all(&out)?;
    }
    fs::create_dir_all(&out)?;

    let processing = bundle_processing(root, &out)?;
    let lygia = bundle_lygia(root, &out)?;
    eprintln!(
        "Bundled {processing} processing and {lygia} lygia modules into {}",
        out.display()
    );
    Ok(())
}
