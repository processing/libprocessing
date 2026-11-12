"""The built-in WESL packages as files, and wiring them into a project's `wesl.toml`
so editor tooling (wgsl-analyzer) resolves `import processing::...` and
`import lygia::...` the same way `Shader` does at runtime."""

import json
import shutil
import sys
from pathlib import Path

from . import processing_py as _native

_BEGIN = "# BEGIN processing_py"
_END = "# END processing_py"
_VENDOR_DIR = Path(".processing_py") / "shaders"


def _default_dir(base):
    # a virtual environment is per project and replaced along with the install
    if sys.prefix != sys.base_prefix:
        return Path(sys.prefix) / "share" / "processing_py" / "shaders"
    return base / _VENDOR_DIR


def shader_packages(dest=None):
    """Write the built-in WESL packages under `dest` and return `{name: directory}`.

    `dest` defaults to the virtual environment, or `.processing_py/shaders` in the
    current directory without one.
    """
    dest = Path(dest).resolve() if dest is not None else _default_dir(Path.cwd())
    written = {}
    for name, files in _native._shader_packages().items():
        target = dest / name
        if target.exists():
            shutil.rmtree(target)
        for rel, contents in files.items():
            path = target / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(contents, encoding="utf-8")
        written[name] = target
    return written


def _toml_path(path, base):
    try:
        p = path.relative_to(base)
    except ValueError:
        p = path
    return json.dumps(p.as_posix())


def _dependency_block(packages, base):
    lines = [_BEGIN]
    for name, path in packages.items():
        lines.append(f"{name} = {{ path = {_toml_path(path, base)} }}")
    lines.append(_END)
    return lines


def _splice(text, block):
    lines = text.splitlines()
    if _BEGIN in lines:
        start = lines.index(_BEGIN)
        try:
            end = lines.index(_END, start)
        except ValueError:
            raise ValueError(f"`{_BEGIN}` without a matching `{_END}`") from None
        lines[start : end + 1] = block
    else:
        header = next(
            (i for i, l in enumerate(lines) if l.split("#", 1)[0].strip() == "[dependencies]"),
            None,
        )
        if header is None:
            if lines and lines[-1].strip():
                lines.append("")
            lines += ["[dependencies]", *block]
        else:
            lines[header + 1 : header + 1] = block
    return "\n".join(lines) + "\n"


def _validate(text, path):
    try:
        import tomllib
    except ImportError:
        return
    try:
        doc = tomllib.loads(text)
    except tomllib.TOMLDecodeError as e:
        raise ValueError(f"updating {path} would produce invalid TOML ({e}); edit it by hand") from None
    if not isinstance(doc.get("dependencies"), dict):
        raise ValueError(f"{path}: `dependencies` is not a table")


def shader_workspace(path=".", vendor=False):
    """Create or update `wesl.toml` in `path` so the built-in `processing` and
    `lygia` packages resolve in the editor.

    The packages are written into the virtual environment. With `vendor=True`,
    or without an environment, they go to `.processing_py/shaders` in `path`,
    for editors that can't see the environment or for checking them in. Rerun
    after upgrading processing-py. Returns the path of the `wesl.toml`.
    """
    base = Path(path).resolve()
    base.mkdir(parents=True, exist_ok=True)
    dest = base / _VENDOR_DIR if vendor else _default_dir(base)
    packages = shader_packages(dest)

    manifest = base / "wesl.toml"
    block = _dependency_block(packages, base)
    if manifest.exists():
        text = _splice(manifest.read_text(encoding="utf-8"), block)
    else:
        header = ['edition = "2026_pre"', 'root = "."']
        # keep the project's own module scan out of the packages
        if dest.is_relative_to(base):
            header.append(f'exclude = ["{dest.relative_to(base).as_posix()}/**"]')
        text = "\n".join([*header, "", "[dependencies]", *block]) + "\n"
    _validate(text, manifest)
    manifest.write_text(text, encoding="utf-8")
    return manifest
