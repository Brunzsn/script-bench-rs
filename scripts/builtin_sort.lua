local item_count = 20000
local modulus = 1000000007
local lcg_modulus = 2147483647
local lcg_multiplier = 48271

function bench()
    local values = {}
    local seed = 54321
    for index = 1, item_count do
        seed = (seed * lcg_multiplier) % lcg_modulus
        values[index] = seed
    end

    table.sort(values, function(left, right)
        return left > right
    end)

    local checksum = 0
    for index = 1, #values do
        checksum = (checksum + values[index] * index) % modulus
    end
    return checksum
end
