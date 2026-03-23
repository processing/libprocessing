from .mewnala import *

import sys as _sys
from . import mewnala as _native
for _name in ("math", "color"):
    _sub = getattr(_native, _name, None)
    if _sub is not None:
        _sys.modules[f"{__name__}.{_name}"] = _sub
del _sys, _native, _name, _sub
