-- Only included if `pxs_utils` feature set. This goes into GLOBAL scope.

function _pxs_items(t)
    -- Get keys and values of a table and return them 
    -- as a table list of {{key, item}, ...}
    local items = {}
    local keys = {}
    for key in pairs(t) do
        table.insert(keys, key)
    end
    table.sort(keys) -- Sort keys to ensure consistent order
    for _, key in ipairs(keys) do
        table.insert(items, { key, t[key] })
    end
    return items
end

