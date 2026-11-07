local count = 100000
local modulus = 1000000007

function bench()
    local values = {}
    for index = 0, count - 1 do
        values[index + 1] = index
    end

    local indexed_sum = 0
    for index = 1, count do
        indexed_sum = indexed_sum + values[index]
    end

    local iterated_sum = 0
    for _, value in ipairs(values) do
        iterated_sum = iterated_sum + value
    end

    return (indexed_sum + iterated_sum) % modulus
end
