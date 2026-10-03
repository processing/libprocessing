# Pycessing

Prototype for python bindings to libprocessing

## To Get Started

### Install venv and maturin 
Follow these [installation instructions](https://pyo3.rs/v0.27.2/getting-started.html)

#### macOS
```bash
brew install glfw
```

### Running code
```
$ maturin develop
#
# ...
#
$ python
>>> import processing
>>> processing.size(500, 500)
```

### Shader editor support

The wheel ships the `processing` and `lygia` WESL packages. To have
[wgsl-analyzer](https://github.com/wgsl-analyzer/wgsl-analyzer) resolve
`import processing::...` and `import lygia::...` in your sketch's shaders:

```
$ python -m mewnala shaders init
```

This writes (or updates) `wesl.toml` in the current directory, pointing at the
installed packages. Pass `--vendor` to copy them into `.mewnala/shaders` instead.
Rerun after upgrading mewnala. In a source checkout, `just py-shaders` generates
the bundle.
