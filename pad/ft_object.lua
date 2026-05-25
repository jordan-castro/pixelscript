local pxs = require('pxs')

local ft_object = {}

function ft_object.function_from_outside()
    pxs.print("Calling from function from outside!")
end

return ft_object