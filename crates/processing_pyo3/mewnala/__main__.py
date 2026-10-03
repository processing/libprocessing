import argparse

from ._shaders import shader_packages, shader_workspace


def main(argv=None):
    parser = argparse.ArgumentParser(prog="python -m mewnala")
    commands = parser.add_subparsers(dest="command", required=True)

    shaders = commands.add_parser("shaders", help="bundled WESL packages")
    shaders_commands = shaders.add_subparsers(dest="shaders_command", required=True)

    init = shaders_commands.add_parser(
        "init", help="create or update wesl.toml so editors resolve processing:: and lygia::"
    )
    init.add_argument("dir", nargs="?", default=".")
    init.add_argument(
        "--vendor", action="store_true", help="copy the packages into DIR/.mewnala/shaders"
    )

    shaders_commands.add_parser("path", help="print where the bundled packages are installed")

    args = parser.parse_args(argv)
    if args.shaders_command == "init":
        print(f"wrote {shader_workspace(args.dir, vendor=args.vendor)}")
    else:
        for name, path in shader_packages().items():
            print(f"{name}\t{path}")


if __name__ == "__main__":
    main()
