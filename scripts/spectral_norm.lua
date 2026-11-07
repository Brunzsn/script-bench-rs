local n = 300
local iterations = 10

local function eval_a(i, j)
    return 1.0 / ((i + j) * (i + j + 1) / 2.0 + i + 1.0)
end

function bench()
    local u = {}
    local v = {}
    for i = 1, n do
        u[i] = 1.0
        v[i] = 0.0
    end

    for _ = 1, iterations do
        for i = 0, n - 1 do
            local sum = 0.0
            for j = 0, n - 1 do
                sum = sum + eval_a(i, j) * u[j + 1]
            end
            v[i + 1] = sum
        end

        for i = 1, n do
            u[i] = 0.0
        end
        for i = 0, n - 1 do
            local value = v[i + 1]
            for j = 0, n - 1 do
                u[j + 1] = u[j + 1] + eval_a(i, j) * value
            end
        end
    end

    for i = 0, n - 1 do
        local sum = 0.0
        for j = 0, n - 1 do
            sum = sum + eval_a(i, j) * u[j + 1]
        end
        v[i + 1] = sum
    end

    local v_bv = 0.0
    local vv = 0.0
    for i = 1, n do
        v_bv = v_bv + u[i] * v[i]
        vv = vv + v[i] * v[i]
    end
    return math.sqrt(v_bv / vv)
end
