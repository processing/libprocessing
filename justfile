default:
    @just --list

py-build:
    cd crates/processing_pyo3 && uv run maturin develop

py-run file:
    cd crates/processing_pyo3 && uv run python ../../{{file}}

wasm-build:
    wasm-pack build crates/processing_wasm --target web --out-dir ../../target/wasm

wasm-release:
    wasm-pack build crates/processing_wasm --target web --out-dir ../../target/wasm --release
    -wasm-opt -Oz target/wasm/processing_wasm_bg.wasm -o target/wasm/processing_wasm_bg.wasm

wasm-serve: wasm-build
    python3 -m http.server 8000
