# Pixel Mods
Pixel Mods is a embeddable scripting library built in Rust.

## Why
Because I spent 3 weeks trying to figure out how to embed pocketpy v2.x in C++17. This way instead I just compile the 
rust library and link it statically.

I specifically use it in my Pixel Ai Dash game, I compile this first into a static library, link it via Scons, and wrap it in GDExtension (godot-cpp).
Then within my C++ extension I expose methods for :
- add_callback() // This will add a callback for all languges you have enabled in cargo features flag.
- add_module() // This will add a module for all languages... you get the idea
- add_variable()
- add_class() // This will add a class/struct/tree/etc whatever the language expects. If the language has none of these, it will create a Pseudo type.

Then you also have the module specific ones:
- module_add_callback()
- module_add_module()
- module_add_variable()
- module_add_class()

And then you can do (import, require, etc whatever the language you plan on using expects.)

## Supported Langs
- LUA *
- Python
- JS
- easyjs

* = default

## Used in
- Pixel Ai Dash

## Future
I will not be maintaining this at all. If it works for me that is it, if you have issues or want to make pull requests, feel free and I will look at them.
But this is not production ready at all. If you use this, it is at your own risk.