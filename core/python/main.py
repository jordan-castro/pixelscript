# Main is always loaded at the start of the Python engine regardless of features. This goes into GLOBAL scope

# A internal register to keep objects alive off the pocketpy stack.
_pxs_register = {}
_pxs_register_next_id = 0

def _pxs_new_register(obj):
    global _pxs_register_next_id
    id = _pxs_register_next_id
    _pxs_register[id] = obj
    _pxs_register_next_id += 1
    return id

