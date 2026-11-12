# processing-py

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
>>> import processing_py
>>> processing_py.size(500, 500)
```

### Shader editor support

The `processing` and `lygia` WESL packages are built into the library. To have
[wgsl-analyzer](https://github.com/wgsl-analyzer/wgsl-analyzer) resolve
`import processing::...` and `import lygia::...` in your sketch's shaders:

```
$ python -m processing_py shaders init
```

This writes the packages into the virtual environment and creates (or updates)
`wesl.toml` in the current directory to point at them. Pass `--vendor`, or run
without an environment, to write them to `.processing_py/shaders` instead.
Rerun after upgrading processing-py.

## Acknowledgements

Thank you to [Faruk Hammoud](https://github.com/FarukHammoud) for generously handing over the `processing-py` name. Their original [processing_py](https://github.com/FarukHammoud/processing_py) project, a Python graphics library based on Processing, was published under this name from 2020 to 2021 (versions 0.1.0–0.3.7).
