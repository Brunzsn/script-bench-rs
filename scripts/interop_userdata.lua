function bench_userdata_method()
    local object = InteropData.new(7)
    local sum = 0
    for _ = 1, 100000 do
        sum = sum + object:get()
    end
    return sum
end

function bench_userdata_field_get()
    local object = InteropData.new(7)
    local sum = 0
    for _ = 1, 100000 do
        sum = sum + object.value
    end
    return sum
end

function bench_userdata_field_set()
    local object = InteropData.new(0)
    for index = 0, 99999 do
        object.value = index
    end
    return object.value
end

function bench_userdata_ctor_copy()
    local count = 0
    for _ = 1, 100000 do
        local object = CopyData.new(7)
        count = count + 1
    end
    return count
end

function bench_userdata_ctor_rc()
    local count = 0
    for _ = 1, 100000 do
        local object = RcData.new("interop")
        count = count + 1
    end
    return count
end
