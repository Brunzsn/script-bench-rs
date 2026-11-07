local count = 20000
local modulus = 1000000007

function bench()
    local map = {}
    for index = 0, count - 1 do
        map["key" .. index] = index * 3
    end

    local looked_up = 0
    for index = 0, count - 1 do
        looked_up = looked_up + map["key" .. index]
    end

    local iterated = 0
    for _, value in pairs(map) do
        iterated = iterated + value
    end

    return (looked_up + iterated + count) % modulus
end
