import processing
import IPython.display as _ipy_display

def _processing_post_execute(result):
    processing._end_draw()
    png_data = processing._readback_png()
    _ipy_display.display(_ipy_display.Image(data=bytes(png_data)))
    processing._begin_draw()

get_ipython().events.register('post_run_cell', _processing_post_execute)
