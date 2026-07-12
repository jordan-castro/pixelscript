# PixelScript Core
This directory contains core language builtins that pixel script exposes and uses internally.

When adding a new language make sure you also add any needed core functions.

## Yoyo
The yoyo core is a C++ library that adds general purpose but optional addons to the pxs runtime.
These can be turned on or off with features.

### Features
To enable yoyo and features, the `yoyo` feature must be enabled.

|name|module|notes|
|----|-------|-----|
|yoyo_full|`yoyo.*`|Adds all modules|
|yoyo_core|`yoyo`|Adds `print`, `println`, `readln`. If you dont add yoyo_core you can still add submodules.|
|yoyo_os|`yoyo.os`|This is cross platform.|
|yoyo_pxs|`yoyo.pxs`|Adds methods for working with `pixelscript`.|
|yoyo_fs|`yoyo.fs`|Adds file system access.|
|yoyo_shell|`yoyo.shell`|Interact with system shell.|
|yoyo_net|`yoyo.net`|Adds low and high level networking/http. Uses `WinHTTP` on windows and `curl` on other platforms.|
|yoyo_zip|`yoyo.zip`|Read/Write/Extract zip files. Requires `yoyo_fs`.|

### Platform support
Current supported platforms in yoyo are:
|name|Windows|Macos|Linux|Android|iOS|
|----|-------|-----|-----|-------|---|
|`yoyo.core`      | Yes | Yes | Yes | Yes | Yes |
|`yoyo.os`        | Yes | Yes | Yes | Yes | Yes |
|`yoyo.pxs`       | Yes | Yes | Yes | Yes | Yes |
|`yoyo.fs`        | Yes | Yes | Yes | Yes | Yes |
|`yoyo.shell`     | Yes | Yes | Yes | Yes | Yes |
|`yoyo.net`       | Yes | No  | No  | No  | No  |
|`yoyo.zip`       | Yes | Yes | Yes | Yes | Yes |
