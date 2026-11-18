import argparse
import os
import sys
import types

from . import _hot
from . import processing_py as _native
from ._shaders import shader_packages, shader_workspace


def run_sketch(sketch, sketch_args, hot_reload):
    path = os.path.abspath(sketch)
    with open(path, "rb") as f:
        code = compile(f.read(), path, "exec")

    if hot_reload:
        os.environ[_hot.ENV_VAR] = "1"

    # mirror what `python sketch.py` sets up
    sys.argv = [path] + sketch_args
    sys.path[0] = os.path.dirname(path)
    module = types.ModuleType("__main__")
    module.__file__ = path
    sys.modules["__main__"] = module

    exec(code, module.__dict__)

    # the sketch may have called run() itself; otherwise call it from the sketch's namespace
    if not getattr(_native, "_run_called", False):
        exec("__import__('processing_py').run()", module.__dict__)


def main(argv=None):
    parser = argparse.ArgumentParser(prog="python -m processing_py")
    commands = parser.add_subparsers(dest="command", required=True)

    run = commands.add_parser("run", help="run a sketch")
    run.add_argument("sketch", help="path to the sketch .py file")
    run.add_argument(
        "--hot-reload",
        action="store_true",
        help="patch edits to the sketch into the running window",
    )

    shaders = commands.add_parser("shaders", help="built-in WESL packages")
    shaders_commands = shaders.add_subparsers(dest="shaders_command", required=True)

    init = shaders_commands.add_parser(
        "init", help="create or update wesl.toml so editors resolve processing:: and lygia::"
    )
    init.add_argument("dir", nargs="?", default=".")
    init.add_argument(
        "--vendor",
        action="store_true",
        help="write the packages to DIR/.processing_py/shaders instead of the virtual environment",
    )

    shaders_commands.add_parser("path", help="write the packages and print where they are")

    # anything `run` doesn't recognise belongs to the sketch
    args, extra = parser.parse_known_args(argv)
    if args.command == "run":
        run_sketch(args.sketch, extra, args.hot_reload)
        return
    if extra:
        parser.error(f"unrecognized arguments: {' '.join(extra)}")

    if args.shaders_command == "init":
        print(f"wrote {shader_workspace(args.dir, vendor=args.vendor)}")
    else:
        for name, path in shader_packages().items():
            print(f"{name}\t{path}")


if __name__ == "__main__":
    main()
