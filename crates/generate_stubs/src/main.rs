use pyo3_introspection::{introspect_cdylib, module_stub_files};
use std::path::{Path, PathBuf};
use std::{env, fs};

fn find_cdylib() -> PathBuf {
    let target_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target")
        .join("release");

    // Platform-specific library name
    let lib_name = if cfg!(target_os = "macos") {
        "libprocessing.dylib"
    } else if cfg!(target_os = "windows") {
        "processing.dll"
    } else {
        "libprocessing.so"
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

    let module = introspect_cdylib(&cdylib_path, "processing").expect("Failed to introspect cdylib");

    let stubs = module_stub_files(&module);

    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("processing_pyo3");

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
