import processing
import IPython.display as _ipy_display

def _processing_post_execute(result):
    processing._present()
    png_data = processing._readback_png()
    if png_data is not None:
        _ipy_display.display(_ipy_display.Image(data=bytes(png_data)))

get_ipython().events.register('post_run_cell', _processing_post_execute)
