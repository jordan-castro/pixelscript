# Only included if `pxs_utils` feature set. This goes into GLOBAL scope

def _pxs_items(d):
    if not type(d) is dict:
        return []
    return list(d.items())

