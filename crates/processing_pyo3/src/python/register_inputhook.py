import mewnala
import time
import traceback
from IPython.terminal.pt_inputhooks import register

def _processing_inputhook(context):
    while not context.input_is_ready():
        if not mewnala._poll_events():
            mewnala._graphics = None
            break
        try:
            mewnala._tick(get_ipython().user_ns)
        except Exception:
            traceback.print_exc()
        time.sleep(1.0 / 60.0)

register('processing', _processing_inputhook)
get_ipython().enable_gui('processing')
