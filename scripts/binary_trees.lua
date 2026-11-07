local depth = 14
local iterations = 20

local function build(level)
    if level == 0 then
        return nil
    end
    return { left = build(level - 1), right = build(level - 1) }
end

local function count(node)
    if node == nil then
        return 1
    end
    return 1 + count(node.left) + count(node.right)
end

function bench()
    local total = 0
    for iteration = 1, iterations do
        total = total + count(build(depth))
    end
    return total
end
