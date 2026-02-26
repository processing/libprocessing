import processing

def _processing_post_execute(result):
    processing._present()
    processing._poll_events()

get_ipython().events.register('post_run_cell', _processing_post_execute)
