set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

export PROCESSING_ASSET_ROOT := canonicalize("./assets")

default:
    @just --list

py-build *args:
    cd crates/processing_pyo3; uv run maturin develop --release {{args}}

py-stubs: py-build
    cargo run --release -p generate_stubs

py-run file *args: (py-build args)
    cd crates/processing_pyo3; uv run python ./examples/{{file}}

py-jupyter file: py-build
    cd crates/processing_pyo3; uv run jupyter notebook ./examples/{{file}}

py-ipython: py-build
    cd crates/processing_pyo3; uv run ipython

wasm-build:
    wasm-pack build crates/processing_wasm --target web --out-dir ../../target/wasm

wasm-release:
    wasm-pack build crates/processing_wasm --target web --out-dir ../../target/wasm --release
    -wasm-opt -Oz target/wasm/processing_wasm_bg.wasm -o target/wasm/processing_wasm_bg.wasm

wasm-serve: wasm-build
    python3 -m http.server 8000
