# PixelScript Core
This directory contains core language builtins that pixel script exposes and uses internally.

When adding a new language make sure you also add any needed core functions.

## Yoyo
The yoyo core is a C++ library that adds general purpose but optional addons to the pxs runtime.
These can be turned on or off with features.

### Features
|name|module|notes|
|----|-------|-----|
|yoyo_core|`yoyo`|Adds `print`, `println`, `readln`. If you dont add yoyo_core you can still add submodules.|
|yoyo_os|`yoyo.os`|This is cross platform.|
|yoyo_pxs|`yoyo.pxs`|Adds methods for working with `pixelscript`.|
|yoyo_fs|`yoyo.fs`|Adds file system access.|
|yoyo_shell|`yoyo.shell`|Interact with system shell.|
|yoyo_net|`yoyo.net`|Adds low and high level networking/http. Uses `WinHTTP` on windows and `curl` on other platforms.|
|yoyo_zip|`yoyo.zip`|Read/Write/Extract zip files.|

### Platform support
Current supported platforms in yoyo are:
- Windows
- MacOS
- Linux
- *Android*
- *iOS*
