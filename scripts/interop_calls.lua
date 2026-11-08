function bench_call_empty()
    local sum = 0
    for _ = 1, 100000 do
        sum = sum + interop_ping()
    end
    return sum
end

function bench_call_one_i64()
    local sum = 0
    for _ = 1, 100000 do
        sum = sum + interop_id_i64(7)
    end
    return sum
end

function bench_call_three_i64()
    local sum = 0
    for _ = 1, 100000 do
        sum = sum + interop_sum_i64_3(1, 2, 3)
    end
    return sum
end

function bench_call_five_i64()
    local sum = 0
    for _ = 1, 100000 do
        sum = sum + interop_sum_i64_5(1, 2, 3, 4, 5)
    end
    return sum
end

function bench_call_f64()
    local sum = 0
    for _ = 1, 100000 do
        sum = sum + interop_id_f64(1.5)
    end
    return sum
end

function bench_call_string_in()
    local sum = 0
    for _ = 1, 100000 do
        sum = sum + interop_strlen("interop")
    end
    return sum
end

function bench_call_string_out()
    local sum = 0
    for _ = 1, 100000 do
        sum = sum + #interop_string_out()
    end
    return sum
end
