let body_count = 5;
let step_count = 100000;
let pi = 3.141592653589793;
let days_per_year = 365.24;
let solar_mass = 4.0 * pi * pi;
let dt = 0.01;

let bodies = [
    #{ x: 0.0, y: 0.0, z: 0.0, vx: 0.0, vy: 0.0, vz: 0.0, mass: solar_mass },
    #{
        x: 4.84143144246472090e+00,
        y: -1.16032004402742839e+00,
        z: -1.03622044471123109e-01,
        vx: 1.66007664274403694e-03 * days_per_year,
        vy: 7.69901118419740425e-03 * days_per_year,
        vz: -6.90460016972063023e-05 * days_per_year,
        mass: 9.54791938424326609e-04 * solar_mass,
    },
    #{
        x: 8.34336671824457987e+00,
        y: 4.12479856412430479e+00,
        z: -4.03523417114321381e-01,
        vx: -2.76742510726862411e-03 * days_per_year,
        vy: 4.99852801234917238e-03 * days_per_year,
        vz: 2.30417297573763929e-05 * days_per_year,
        mass: 2.85885980666130812e-04 * solar_mass,
    },
    #{
        x: 1.28943695621391310e+01,
        y: -1.51111514016986312e+01,
        z: -2.23307578892655734e-01,
        vx: 2.96460137564761618e-03 * days_per_year,
        vy: 2.37847173959480950e-03 * days_per_year,
        vz: -2.96589568540237556e-05 * days_per_year,
        mass: 4.36624404335156298e-05 * solar_mass,
    },
    #{
        x: 1.53796971148509165e+01,
        y: -2.59193146099879641e+01,
        z: 1.79258772950371181e-01,
        vx: 2.68067772490389322e-03 * days_per_year,
        vy: 1.62824170038242295e-03 * days_per_year,
        vz: -9.51592254519715870e-05 * days_per_year,
        mass: 5.15138902046611451e-05 * solar_mass,
    },
];

for _step in 0..step_count {
    for i in 0..body_count {
        for j in (i + 1)..body_count {
            let dx = bodies[i].x - bodies[j].x;
            let dy = bodies[i].y - bodies[j].y;
            let dz = bodies[i].z - bodies[j].z;
            let distance_squared = dx * dx + dy * dy + dz * dz;
            let magnitude = dt / (distance_squared * distance_squared.sqrt());
            let mass_i = bodies[i].mass;
            let mass_j = bodies[j].mass;

            bodies[i].vx -= dx * mass_j * magnitude;
            bodies[i].vy -= dy * mass_j * magnitude;
            bodies[i].vz -= dz * mass_j * magnitude;
            bodies[j].vx += dx * mass_i * magnitude;
            bodies[j].vy += dy * mass_i * magnitude;
            bodies[j].vz += dz * mass_i * magnitude;
        }
    }

    for i in 0..body_count {
        bodies[i].x += dt * bodies[i].vx;
        bodies[i].y += dt * bodies[i].vy;
        bodies[i].z += dt * bodies[i].vz;
    }
}

let energy = 0.0;
for i in 0..body_count {
    energy += 0.5 * bodies[i].mass
        * (bodies[i].vx * bodies[i].vx + bodies[i].vy * bodies[i].vy + bodies[i].vz * bodies[i].vz);
}
for i in 0..body_count {
    for j in (i + 1)..body_count {
        let dx = bodies[i].x - bodies[j].x;
        let dy = bodies[i].y - bodies[j].y;
        let dz = bodies[i].z - bodies[j].z;
        energy -= bodies[i].mass * bodies[j].mass / (dx * dx + dy * dy + dz * dz).sqrt();
    }
}

energy
