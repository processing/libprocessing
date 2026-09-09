use std::{env, path::PathBuf};

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let output_dir = PathBuf::from(&crate_dir).join("include");

    std::fs::create_dir_all(&output_dir).expect("Failed to create include directory");

    let output_file = output_dir.join("processing.h");
    let config_path = PathBuf::from(&crate_dir).join("cbindgen.toml");

    cbindgen::Builder::new()
        .with_config(
            cbindgen::Config::from_file(&config_path).expect("Failed to load cbindgen.toml"),
        )
        .with_crate(&crate_dir)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(&output_file);

    // The whole source tree, not just lib.rs: exports live in modules
    // (video.rs, canvas.rs, ...) too, and naming only lib.rs meant a new or
    // changed export in any of them left a stale header behind -- which shows
    // up much later as a jextract binding that does not match the library.
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=cbindgen.toml");
    println!(
        "cargo:warning=Generated header at: {}",
        output_file.display()
    );
}
