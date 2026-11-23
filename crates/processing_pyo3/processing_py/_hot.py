import os
import time

ENV_VAR = "PROCESSING_HOT_RELOAD"

_ON = ("1", "true", "yes", "on")
_OFF = ("0", "false", "no", "off", "")

# seconds a changed file must sit untouched, so a half-written save is never parsed
_SETTLE = 0.05


def _stamp(filename):
    try:
        st = os.stat(filename)
    except OSError:
        return None
    return (st.st_mtime_ns, st.st_size)


class HotReloader:

    def __init__(self, registry, log, sketch_globals, sketch_file):
        self._registry = registry
        self._log = log
        self._globals = sketch_globals
        self._sketch_file = sketch_file
        self._stamps = {}
        self._pending = {}
        registry.set_logger(log)
        registry.precache_activity.register(self._on_prepare)

    def _on_prepare(self, module_name, filename):
        self._stamps[filename] = _stamp(filename)

    def _setup_code(self):
        code = getattr(self._globals.get("setup"), "__code__", None)
        if code is None:
            return None
        # bytecode only, so comments and blank lines do not count as a change
        return (code.co_code, code.co_consts, code.co_names)

    def poll(self):
        now = time.monotonic()
        for filename, old in list(self._stamps.items()):
            new = _stamp(filename)
            # an empty file is a save in progress, not an edit
            if new is None or new == old or new[1] == 0:
                continue
            self._stamps[filename] = new
            self._pending[filename] = now

        ready = [f for f, t in self._pending.items() if now - t >= _SETTLE]
        if not ready:
            return None

        # the sketch goes last, so anything it newly imports already exists
        ready.sort(key=lambda f: f == self._sketch_file)

        before = self._setup_code()
        for filename in ready:
            del self._pending[filename]
            try:
                codefile = self._registry.get(filename)
                if codefile is not None:
                    codefile.refresh()
            except Exception as exc:
                self._log(exc)
        after = self._setup_code()
        return after is not None and after != before


def enabled(requested):
    value = os.environ.get(ENV_VAR)
    if value is None:
        return bool(requested)
    value = value.strip().lower()
    if value in _ON:
        return True
    if value in _OFF:
        return False
    raise RuntimeError(f"{ENV_VAR}={value!r} is not a boolean; use 1 or 0")


def _in_sketch_dir(sketch_file):
    root = os.path.dirname(os.path.realpath(sketch_file)) + os.sep

    def matches(filename):
        path = os.path.realpath(filename)
        return path.startswith(root) and "site-packages" not in path

    return matches


def start(sketch_globals, requested=None):
    if not enabled(requested):
        return None

    filename = sketch_globals.get("__file__")
    module_name = sketch_globals.get("__name__")
    if not filename or not module_name or not os.path.isfile(filename):
        raise RuntimeError("hot reload needs a sketch that was run from a .py file")
    from codefind import code_registry
    from jurigged.live import default_logger
    from jurigged.register import registry

    # __main__'s functions bypass codefind's exec hook, so collect them now
    code_registry.collect_all()

    reloader = HotReloader(registry, default_logger, sketch_globals, filename)
    registry.prepare(module_name, filename)
    # also watch the sketch's sibling modules
    registry.auto_register(filter=_in_sketch_dir(filename))
    return reloader
