-- Create a table to hold your "exports"
local ft_object = {}

function ft_object.function_from_outside()
    println("Calling from function from outside!")
end

-- Return the table so 'require' can give it to your main script
return ft_object