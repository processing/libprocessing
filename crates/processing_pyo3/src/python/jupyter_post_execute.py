import mewnala
import IPython.display as _ipy_display

def _processing_post_execute(result):
    mewnala._present()
    mewnala._tick(get_ipython().user_ns)
    png_data = mewnala._readback_png()
    if png_data is not None:
        _ipy_display.display(_ipy_display.Image(data=bytes(png_data)))

get_ipython().events.register('post_run_cell', _processing_post_execute)
