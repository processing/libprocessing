import processing
import IPython.display as _ipy_display

def _processing_post_execute(result):
    if getattr(processing, '_graphics', None) is None:
        return
    processing._present()
    png_data = processing._readback_png()
    _ipy_display.display(_ipy_display.Image(data=bytes(png_data)))

get_ipython().events.register('post_run_cell', _processing_post_execute)
