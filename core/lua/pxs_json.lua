-- Only included if `pxs_json` feature.

-- Import __dkjson__ lib
local __dkjson__ = require("__dkjson__")

local pxs_json = {}

-- Encode a Tree into a JSON string.
function pxs_json.encode(object)
    return __dkjson__.encode(object)
end

-- Decode a string into a Tree
function pxs_json.decode(string)
    return __dkjson__.decode(string, 1, nil)
end

return pxs_json