import processing
import time
from IPython.terminal.pt_inputhooks import register

def _processing_inputhook(context):
    while not context.input_is_ready():
        processing._poll_events()
        time.sleep(1.0 / 60.0)

register('processing', _processing_inputhook)
get_ipython().enable_gui('processing')
