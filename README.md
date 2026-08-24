# libprocessing

> [!WARNING]
> This project is very much R&D and highly unstable

libprocessing is an experimental native library with the goal of supporting the implementation of the core Processing API in a variety of languages. The library is written in the [Rust programming language](https://rust-lang.org/) and built on top of the [Bevy game engine](https://bevy.org/). libprocessing uses [WebGPU](https://webgpu.org/) as its rendering backend and is designed to (eventually) support desktop, mobile, and web targets.

You can learn more about this project from this [talk at LibreGraphicsMeeting 2026](https://app.media.ccc.de/v/lgm-2026-110668-expanding-processing-s-future-with-a-rust-rendering-engine)

## Getting started

There is two different things at work here :
- libprocessing (this repo) is the low-level cross-platform library for the core Processing API. It's written in Rust, and thus lets write Processing sketches in Rust.
- mewnala is a Python package built directly from libprocessing, providing Python bindings for the library. It's available as any other Python package out there and lets you write Processing sketches in a Python environment, regardless of you having libprocessing or Rust installed.

### mewnala (the python library)

Inside of our `processing_pyo3` crate we have created a python package that you can easily install with pip.
Again, we are still very nascent, but let us know what kinds of snags you may run into while getting this set up.
Try running the examples in the [processing_pyo3 examples directory](crates/processing_pyo3/examples).

We are big fans of [uv](https://github.com/astral-sh/uv) and this is the easiest way to get started using `mewnala`

#### Setting up uv on linux or macOS
```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
```

#### For Windows users:
```bash
powershell -ExecutionPolicy ByPass -c "irm https://astral.sh/uv/install.ps1 | iex"
```

#### Install a mewnala

We're going to create a folder named `mewnala-sketchbook` and install the `mewnala` package inside:

```bash
# Initialize a project with uv
uv init mewnala-sketchbook && cd mewnala-sketchbook

# add the package
uv add mewnala
```

Now create a file named `sketchh.py` at the root of `mewnala-sketchbook`.

You can use this code to test

```python
from mewnala import *

def setup():
    size(400, 400)
    background(255)

def draw():
    push_matrix()
    translate(mouse_x, mouse_y)
    fill(255, 100, 200)
    circle(0, 0, 50)
    pop_matrix()

run()
```

Now run your sketch using:

```bash
# run a sketch
uv run sketch.py
```

_Note: you can use any file name you want for your sketch_

### Rust (libprocessing)

You'll need to install the Rust toolchain to work on this project. Most users will want to install Rust via [`rustup`](https://rustup.rs/), which helps manage Rust toolchain versions.

### Clone the project and its submodules

When cloning this repo (or your fork), don't forget to install its submodules (eg. Lygia)

```bash
git clone --recurse-submodules git@github.com:processing/libprocessing.git
```

if you already cloned the repo without `--recurse-submodules`, you can run

```bash
git submodule update --init
```

### Build commands

This project uses [just](https://github.com/casey/just) as a command runner:

```bash
cargo install just
```

Run `just` to see available commands.

## Building for web

The `processing_wasm` crate provides WebAssembly bindings that expose a JavaScript API mirroring the C FFI.

### Requirements

Install [wasm-pack](https://rustwasm.github.io/wasm-pack/):

```bash
cargo install wasm-pack
```

You'll also need the wasm32 target:

```bash
rustup target add wasm32-unknown-unknown
```

### Build

```bash
just wasm-build
```

This outputs the package to `target/wasm/`.

### Run the example

```bash
just wasm-serve
```
_Note: you'll need python installed on your machine to run this command_


## Contributing

We want your help building this library! 

One place we could really use some help is with porting Processing examples to using mewnala, and then reporting where you get stuck or where things could be easier. Check out the effort here: https://github.com/processing/processing-examples-mewnala. 


You can see a list of outstanding tasks in our [issues](https://github.com/processing/libprocessing). However, while we're still in the early phases, consider checking in with us first in the `#devs-chat` channel on [Discord](https://discord.gg/h99u95nU7q) to coordinate our efforts.

You can read our project design principles [here](./docs/principles.md).
