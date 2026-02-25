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

    wesl::PkgBuilder::new("lygia")
        .scan_root("../../lygia")
        .expect("failed to scan Lygia WESL files")
        .build_artifact()
        .expect("failed to build Lygia package artifact");
}
