function bench_bulk_f64_in_list(values)
    local sum = 0
    for index = 1, #values do
        sum = sum + values[index]
    end
    return sum
end

function bench_bulk_u8_in_list(values)
    local sum = 0
    for index = 1, #values do
        sum = sum + values[index]
    end
    return sum
end

local function make_unit_list(size, value)
    local values = {}
    for index = 1, size do
        values[index] = value
    end
    return values
end

local f64_list_1k = make_unit_list(1000, 1.0)
local f64_list_100k = make_unit_list(100000, 1.0)
local f64_list_1m = make_unit_list(1000000, 1.0)
local u8_list_1k = make_unit_list(1000, 1)
local u8_list_100k = make_unit_list(100000, 1)
local u8_list_1m = make_unit_list(1000000, 1)

function bench_bulk_f64_out_list_1k()
    return interop_sum_f64_list(f64_list_1k)
end

function bench_bulk_f64_out_list_100k()
    return interop_sum_f64_list(f64_list_100k)
end

function bench_bulk_f64_out_list_1m()
    return interop_sum_f64_list(f64_list_1m)
end

function bench_bulk_u8_out_list_1k()
    return interop_sum_u8_list(u8_list_1k)
end

function bench_bulk_u8_out_list_100k()
    return interop_sum_u8_list(u8_list_100k)
end

function bench_bulk_u8_out_list_1m()
    return interop_sum_u8_list(u8_list_1m)
end
