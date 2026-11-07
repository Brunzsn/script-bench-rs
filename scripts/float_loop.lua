function bench()
    local value = 1.0
    for index = 0, 4999999 do
        value = value * 0.999999 + 0.000001
    end
    return value
end
