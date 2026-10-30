import processing_py
import IPython.display as _ipy_display

def _processing_post_execute(result):
    processing_py._present()
    processing_py._tick(get_ipython().user_ns)
    png_data = processing_py._readback_png()
    if png_data is not None:
        _ipy_display.display(_ipy_display.Image(data=bytes(png_data)))

get_ipython().events.register('post_run_cell', _processing_post_execute)
