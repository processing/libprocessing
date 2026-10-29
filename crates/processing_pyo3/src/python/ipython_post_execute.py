import processing_py

def _processing_post_execute(result):
    processing_py._present()
    if not processing_py._poll_events():
        processing_py._graphics = None
        return
    processing_py._tick(get_ipython().user_ns)

get_ipython().events.register('post_run_cell', _processing_post_execute)
