function bench()
    local modulus = 1000000007
    local value = 12345
    for index = 0, 9999999 do
        value = (value * 31 + index) % modulus
    end
    return value
end
