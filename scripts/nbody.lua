local body_count = 5
local step_count = 100000
local pi = 3.141592653589793
local days_per_year = 365.24
local solar_mass = 4.0 * pi * pi
local dt = 0.01

local function body(x, y, z, vx, vy, vz, mass)
    return { x = x, y = y, z = z, vx = vx, vy = vy, vz = vz, mass = mass }
end

function bench()
    local bodies = {
        body(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, solar_mass),
        body(
            4.84143144246472090e+00,
            -1.16032004402742839e+00,
            -1.03622044471123109e-01,
            1.66007664274403694e-03 * days_per_year,
            7.69901118419740425e-03 * days_per_year,
            -6.90460016972063023e-05 * days_per_year,
            9.54791938424326609e-04 * solar_mass
        ),
        body(
            8.34336671824457987e+00,
            4.12479856412430479e+00,
            -4.03523417114321381e-01,
            -2.76742510726862411e-03 * days_per_year,
            4.99852801234917238e-03 * days_per_year,
            2.30417297573763929e-05 * days_per_year,
            2.85885980666130812e-04 * solar_mass
        ),
        body(
            1.28943695621391310e+01,
            -1.51111514016986312e+01,
            -2.23307578892655734e-01,
            2.96460137564761618e-03 * days_per_year,
            2.37847173959480950e-03 * days_per_year,
            -2.96589568540237556e-05 * days_per_year,
            4.36624404335156298e-05 * solar_mass
        ),
        body(
            1.53796971148509165e+01,
            -2.59193146099879641e+01,
            1.79258772950371181e-01,
            2.68067772490389322e-03 * days_per_year,
            1.62824170038242295e-03 * days_per_year,
            -9.51592254519715870e-05 * days_per_year,
            5.15138902046611451e-05 * solar_mass
        )
    }

    for _ = 1, step_count do
        for i = 1, body_count do
            for j = i + 1, body_count do
                local dx = bodies[i].x - bodies[j].x
                local dy = bodies[i].y - bodies[j].y
                local dz = bodies[i].z - bodies[j].z
                local distance_squared = dx * dx + dy * dy + dz * dz
                local magnitude = dt / (distance_squared * math.sqrt(distance_squared))
                local mass_i = bodies[i].mass
                local mass_j = bodies[j].mass

                bodies[i].vx = bodies[i].vx - dx * mass_j * magnitude
                bodies[i].vy = bodies[i].vy - dy * mass_j * magnitude
                bodies[i].vz = bodies[i].vz - dz * mass_j * magnitude
                bodies[j].vx = bodies[j].vx + dx * mass_i * magnitude
                bodies[j].vy = bodies[j].vy + dy * mass_i * magnitude
                bodies[j].vz = bodies[j].vz + dz * mass_i * magnitude
            end
        end

        for i = 1, body_count do
            bodies[i].x = bodies[i].x + dt * bodies[i].vx
            bodies[i].y = bodies[i].y + dt * bodies[i].vy
            bodies[i].z = bodies[i].z + dt * bodies[i].vz
        end
    end

    local energy = 0.0
    for i = 1, body_count do
        energy = energy + 0.5 * bodies[i].mass
            * (bodies[i].vx * bodies[i].vx + bodies[i].vy * bodies[i].vy + bodies[i].vz * bodies[i].vz)
    end
    for i = 1, body_count do
        for j = i + 1, body_count do
            local dx = bodies[i].x - bodies[j].x
            local dy = bodies[i].y - bodies[j].y
            local dz = bodies[i].z - bodies[j].z
            energy = energy - bodies[i].mass * bodies[j].mass / math.sqrt(dx * dx + dy * dy + dz * dz)
        end
    end

    return energy
end
