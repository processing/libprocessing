"""WESL packages bundled with mewnala, and wiring them into a project's `wesl.toml`
so editor tooling (wgsl-analyzer) resolves `import processing::...` and
`import lygia::...` the same way `Shader` does at runtime."""

import json
import shutil
from pathlib import Path

PACKAGES = ("processing", "lygia")

_BEGIN = "# BEGIN mewnala"
_END = "# END mewnala"
_VENDOR_DIR = Path(".mewnala") / "shaders"


def shader_packages():
    """Return `{name: directory}` for the WESL packages shipped in the wheel."""
    root = Path(__file__).parent / "shaders"
    found = {name: root / name for name in PACKAGES if (root / name / "wesl.toml").is_file()}
    if not found:
        raise FileNotFoundError(
            f"no bundled shader packages under {root}; "
            "in a source checkout run `cargo run -p bundle_shaders`"
        )
    return found


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
    """Create or update `wesl.toml` in `path` so the bundled `processing` and
    `lygia` packages resolve in the editor.

    Dependencies point into the installed package. With `vendor=True` they are
    copied to `.mewnala/shaders` first, for editors that can't see the
    environment's site-packages or for checking them in. Rerun after upgrading
    mewnala either way. Returns the path of the `wesl.toml`.
    """
    base = Path(path).resolve()
    base.mkdir(parents=True, exist_ok=True)
    packages = shader_packages()

    if vendor:
        dst = base / _VENDOR_DIR
        vendored = {}
        for name, src in packages.items():
            target = dst / name
            if target.exists():
                shutil.rmtree(target)
            shutil.copytree(src, target)
            vendored[name] = target
        packages = vendored

    manifest = base / "wesl.toml"
    block = _dependency_block(packages, base)
    if manifest.exists():
        text = _splice(manifest.read_text(encoding="utf-8"), block)
    else:
        header = ['edition = "2026_pre"', 'root = "."']
        if vendor:
            header.append(f'exclude = ["{_VENDOR_DIR.as_posix()}/**"]')
        text = "\n".join([*header, "", "[dependencies]", *block]) + "\n"
    _validate(text, manifest)
    manifest.write_text(text, encoding="utf-8")
    return manifest
