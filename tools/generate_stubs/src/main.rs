use pyo3_introspection::{introspect_cdylib, module_stub_files};
use std::path::{Path, PathBuf};
use std::{env, fs};

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
}

fn find_cdylib() -> PathBuf {
    let target_dir = workspace_root().join("target").join("release");

    // Platform-specific library name
    let lib_name = if cfg!(target_os = "macos") {
        "libmewnala.dylib"
    } else if cfg!(target_os = "windows") {
        "mewnala.dll"
    } else {
        "libmewnala.so"
    };

    let path = target_dir.join(lib_name);
    if !path.exists() {
        eprintln!("Could not find {}", path.display());
        eprintln!("Make sure to build processing_pyo3 first:");
        eprintln!("  cargo build --release -p processing_pyo3");
        std::process::exit(1);
    }
    path
}

fn main() {
    let cdylib_path = if let Some(path) = env::args().nth(1) {
        PathBuf::from(path)
    } else {
        find_cdylib()
    };

    eprintln!("Introspecting: {}", cdylib_path.display());

    let mut module =
        introspect_cdylib(&cdylib_path, "mewnala").expect("Failed to introspect cdylib");

    module.incomplete = false;

    let stubs = module_stub_files(&module);

    let output_dir = workspace_root()
        .join("crates")
        .join("processing_pyo3")
        .join("mewnala");

    for (filename, content) in &stubs {
        let out_path = output_dir.join(filename);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&out_path, content).unwrap();
        eprintln!("Wrote: {}", out_path.display());
    }

    eprintln!("Done! Generated {} stub file(s)", stubs.len());
}
