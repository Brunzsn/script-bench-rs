local item_count = 10000
local modulus = 1000000007
local lcg_modulus = 2147483647
local lcg_multiplier = 48271

local function partition(values, lo, hi)
    local pivot_index = math.floor((lo + hi) / 2)
    values[pivot_index], values[hi] = values[hi], values[pivot_index]
    local pivot = values[hi]
    local split = lo
    for index = lo, hi - 1 do
        if values[index] < pivot then
            values[index], values[split] = values[split], values[index]
            split = split + 1
        end
    end
    values[split], values[hi] = values[hi], values[split]
    return split
end

local function quicksort(values, lo, hi)
    while lo < hi do
        local pivot = partition(values, lo, hi)
        if pivot > lo then
            quicksort(values, lo, pivot - 1)
        end
        lo = pivot + 1
    end
end

function bench()
    local values = {}
    local seed = 12345
    for index = 1, item_count do
        seed = (seed * lcg_multiplier) % lcg_modulus
        values[index] = seed
    end

    quicksort(values, 1, #values)

    local checksum = 0
    for index = 1, #values do
        checksum = (checksum + values[index] * index) % modulus
    end
    return checksum
end
