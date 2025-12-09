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
