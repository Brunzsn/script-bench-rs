function bench()
    local size = 400
    local size_float = 400.0
    local max_iterations = 50.0
    local checksum = 0.0

    for y = 0, size - 1 do
        local ci = 2.0 * y / size_float - 1.0
        for x = 0, size - 1 do
            local cr = 2.0 * x / size_float - 1.5
            local zr = 0.0
            local zi = 0.0
            local iteration = 0.0
            while iteration < max_iterations and zr * zr + zi * zi <= 4.0 do
                local next_zr = zr * zr - zi * zi + cr
                zi = 2.0 * zr * zi + ci
                zr = next_zr
                iteration = iteration + 1.0
            end
            checksum = checksum + iteration
        end
    end

    return checksum
end
