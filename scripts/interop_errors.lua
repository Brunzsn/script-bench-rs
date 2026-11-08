function bench_error_caught()
    local count = 0
    for _ = 1, 10000 do
        local ok = pcall(interop_fail)
        if not ok then
            count = count + 1
        end
    end
    return count
end

function bench_error_propagates()
    interop_fail()
end
