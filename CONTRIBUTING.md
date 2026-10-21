# Contributing to PixelScript

This is a guide on how to contribute to pixelscript.

## CoreLib
When developing the corelib, pixelscript sticks to a specific documentation system.

Documentation is generated automatically via `scripts/make_corelib.py`

### @pxs
- `@pxs` needs to be added to every single function wanting to be exported to the docs.
- `()` can be used to define naming, i.e. `@pxs(File)`.

### @enum
- `@enum` should be added to enums only.
- `@values(N.. = ..)` are required if you want to specify to the user what enum values are used.

### @self
- `@self` is used when a classmethod function requires a instance to itself as the first argument.

A little redundant if you are already using `@classmethod` but it's kept here in case static methods are added in the future.

### @classmethod
- `@classmethod(#``name``)` adds indentation, making it obvious this function is a child of the parent.

``name`` should be the parent class.

### @except
- `@except` defines that XYZ function can throw a exception.

### @args
- `@args(name:type(comment)))` for defining arguments. Types should be pxs_VarType names or in ` #``CustomTypeName`` ` format. args are comma delimited

### @return
- `@return(type) comment` determines the return type. Types should be pxs_VarType names or in ` #``CustomTypeName`` ` format.
