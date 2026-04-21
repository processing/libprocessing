import mewnala

def _processing_post_execute(result):
    mewnala._present()
    if not mewnala._poll_events():
        mewnala._graphics = None
        return
    mewnala._tick(get_ipython().user_ns)

get_ipython().events.register('post_run_cell', _processing_post_execute)
