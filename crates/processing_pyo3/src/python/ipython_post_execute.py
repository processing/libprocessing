import processing

def _processing_post_execute(result):
    processing._present()
    if not processing._poll_events():
        processing._graphics = None

get_ipython().events.register('post_run_cell', _processing_post_execute)
