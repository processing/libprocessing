def _detect_env():
    try:
        shell = get_ipython().__class__.__name__
        if shell == 'ZMQInteractiveShell':
            return 'jupyter'
        elif shell == 'TerminalInteractiveShell':
            return 'ipython'
    except NameError:
        pass
    return 'script'
_env = _detect_env()
