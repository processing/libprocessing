use std::path::Path;

// packages wesl libraries for processing and lygia
// note that bevy does not (as of yet) understand how to use these packages
// they are, instead, used by us as part of shader pre-processing in the
// processing_render crate
fn main() {
    wesl::PkgBuilder::new("processing")
        .scan_root("shaders/processing")
        .expect("failed to scan processing WESL files")
        .build_artifact()
        .expect("failed to build processing package artifact");

    let mut lygia = wesl::PkgBuilder::new("lygia")
        .scan_root("../../lygia")
        .expect("failed to scan Lygia WESL files");
    wesl_only(&mut lygia.root, Path::new("../../lygia"));
    lygia.root.submodules.retain(|sub| sub.name != "test");
    lygia
        .build_artifact()
        .expect("failed to build Lygia package artifact");
}

// lygia's manifest includes `**/*.wesl` only; the scan also picks up its legacy `.wgsl`
fn wesl_only(module: &mut wesl::Module, path: &Path) {
    if !path.with_extension("wesl").is_file() {
        module.source.clear();
    }
    for sub in &mut module.submodules {
        wesl_only(sub, &path.join(&sub.name));
    }
    module
        .submodules
        .retain(|sub| !sub.source.is_empty() || !sub.submodules.is_empty());
}
