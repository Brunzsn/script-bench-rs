function bench_bulk_f64_in_buffer(values)
    local sum = 0
    local length = buffer.len(values)
    for index = 0, length / 8 - 1 do
        sum = sum + buffer.readf64(values, index * 8)
    end
    return sum
end

function bench_bulk_u8_in_buffer(values)
    local sum = 0
    local length = buffer.len(values)
    for index = 0, length - 1 do
        sum = sum + buffer.readu8(values, index)
    end
    return sum
end

local function make_f64_buffer(size)
    local values = buffer.create(size * 8)
    for index = 0, size - 1 do
        buffer.writef64(values, index * 8, 1.0)
    end
    return values
end

local function make_u8_buffer(size)
    local values = buffer.create(size)
    for index = 0, size - 1 do
        buffer.writeu8(values, index, 1)
    end
    return values
end

local f64_buffer_1k = make_f64_buffer(1000)
local f64_buffer_100k = make_f64_buffer(100000)
local f64_buffer_1m = make_f64_buffer(1000000)
local u8_buffer_1k = make_u8_buffer(1000)
local u8_buffer_100k = make_u8_buffer(100000)
local u8_buffer_1m = make_u8_buffer(1000000)

function bench_bulk_f64_out_buffer_1k()
    return interop_sum_f64_buffer(f64_buffer_1k)
end

function bench_bulk_f64_out_buffer_100k()
    return interop_sum_f64_buffer(f64_buffer_100k)
end

function bench_bulk_f64_out_buffer_1m()
    return interop_sum_f64_buffer(f64_buffer_1m)
end

function bench_bulk_u8_out_buffer_1k()
    return interop_sum_u8_buffer(u8_buffer_1k)
end

function bench_bulk_u8_out_buffer_100k()
    return interop_sum_u8_buffer(u8_buffer_100k)
end

function bench_bulk_u8_out_buffer_1m()
    return interop_sum_u8_buffer(u8_buffer_1m)
end
