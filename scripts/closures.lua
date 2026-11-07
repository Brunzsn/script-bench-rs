local count = 10000
local passes = 50
local modulus = 1000000007

local function map(values, transform)
    local result = {}
    for index = 1, #values do
        result[index] = transform(values[index])
    end
    return result
end

local function filter(values, predicate)
    local result = {}
    for index = 1, #values do
        if predicate(values[index]) then
            table.insert(result, values[index])
        end
    end
    return result
end

local function reduce(values, combine, initial)
    local accumulator = initial
    for index = 1, #values do
        accumulator = combine(accumulator, values[index])
    end
    return accumulator
end

function bench()
    local values = {}
    for index = 0, count - 1 do
        values[index + 1] = index
    end

    local checksum = 0
    for _ = 1, passes do
        local mapped = map(values, function(value)
            return value * 2
        end)
        local filtered = filter(mapped, function(value)
            return value % 3 == 0
        end)
        local sum = reduce(filtered, function(accumulator, value)
            return accumulator + value
        end, 0)
        checksum = (checksum + sum) % modulus
    end
    return checksum
end
