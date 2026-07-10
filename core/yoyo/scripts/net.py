# yoyo.net scripting helper (not intended to be used outside of the `yoyo` core)


def yoyo_get_host_and_path(url:str) -> list[str]:
    res = []
    if len(url) == 0:
        return res
    
    paths = url.split("/")

    if len(paths) >= 3:
        # host
        res.append(paths[2])
        # path
        res.append("/".join(paths[3:]))
    return res
    
