from pathlib import Path

import mkdocs_gen_files

readme = Path("crates/processing_pyo3/README.md").read_text(encoding="utf-8")
with mkdocs_gen_files.open("index.md", "w") as f:
    f.write(readme)

modules = {
    "reference/index.md": ("API Reference", "mewnala", {"show_submodules": False}),
    "reference/math.md": ("mewnala.math", "mewnala.math", {}),
    "reference/color.md": ("mewnala.color", "mewnala.color", {}),
}

for path, (title, module, options) in modules.items():
    with mkdocs_gen_files.open(path, "w") as f:
        f.write(f"# {title}\n\n")
        if options:
            opts = "\n".join(f"      {k}: {v}" for k, v in options.items())
            f.write(f"::: {module}\n    options:\n{opts}\n")
        else:
            f.write(f"::: {module}\n")
