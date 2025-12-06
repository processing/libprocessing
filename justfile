default:
    @just --list

wasm-build:
    wasm-pack build crates/processing_wasm --target web --out-dir ../../target/wasm

wasm-release:
    wasm-pack build crates/processing_wasm --target web --out-dir ../../target/wasm --release
    -wasm-opt -Oz target/wasm/processing_wasm_bg.wasm -o target/wasm/processing_wasm_bg.wasm

wasm-serve: wasm-build
    python3 -m http.server 8000
