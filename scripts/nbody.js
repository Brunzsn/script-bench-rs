const bodyCount = 5;
const stepCount = 100000;
const pi = 3.141592653589793;
const daysPerYear = 365.24;
const solarMass = 4.0 * pi * pi;
const dt = 0.01;

function body(x, y, z, vx, vy, vz, mass) {
    return { x: x, y: y, z: z, vx: vx, vy: vy, vz: vz, mass: mass };
}

function bench() {
    const bodies = [
        body(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, solarMass),
        body(
            4.84143144246472090e+00,
            -1.16032004402742839e+00,
            -1.03622044471123109e-01,
            1.66007664274403694e-03 * daysPerYear,
            7.69901118419740425e-03 * daysPerYear,
            -6.90460016972063023e-05 * daysPerYear,
            9.54791938424326609e-04 * solarMass
        ),
        body(
            8.34336671824457987e+00,
            4.12479856412430479e+00,
            -4.03523417114321381e-01,
            -2.76742510726862411e-03 * daysPerYear,
            4.99852801234917238e-03 * daysPerYear,
            2.30417297573763929e-05 * daysPerYear,
            2.85885980666130812e-04 * solarMass
        ),
        body(
            1.28943695621391310e+01,
            -1.51111514016986312e+01,
            -2.23307578892655734e-01,
            2.96460137564761618e-03 * daysPerYear,
            2.37847173959480950e-03 * daysPerYear,
            -2.96589568540237556e-05 * daysPerYear,
            4.36624404335156298e-05 * solarMass
        ),
        body(
            1.53796971148509165e+01,
            -2.59193146099879641e+01,
            1.79258772950371181e-01,
            2.68067772490389322e-03 * daysPerYear,
            1.62824170038242295e-03 * daysPerYear,
            -9.51592254519715870e-05 * daysPerYear,
            5.15138902046611451e-05 * solarMass
        )
    ];

    for (let step = 0; step < stepCount; step++) {
        for (let i = 0; i < bodyCount; i++) {
            for (let j = i + 1; j < bodyCount; j++) {
                const dx = bodies[i].x - bodies[j].x;
                const dy = bodies[i].y - bodies[j].y;
                const dz = bodies[i].z - bodies[j].z;
                const distanceSquared = dx * dx + dy * dy + dz * dz;
                const magnitude = dt / (distanceSquared * Math.sqrt(distanceSquared));
                const massI = bodies[i].mass;
                const massJ = bodies[j].mass;

                bodies[i].vx -= dx * massJ * magnitude;
                bodies[i].vy -= dy * massJ * magnitude;
                bodies[i].vz -= dz * massJ * magnitude;
                bodies[j].vx += dx * massI * magnitude;
                bodies[j].vy += dy * massI * magnitude;
                bodies[j].vz += dz * massI * magnitude;
            }
        }

        for (let i = 0; i < bodyCount; i++) {
            bodies[i].x += dt * bodies[i].vx;
            bodies[i].y += dt * bodies[i].vy;
            bodies[i].z += dt * bodies[i].vz;
        }
    }

    let energy = 0.0;
    for (let i = 0; i < bodyCount; i++) {
        energy += 0.5 * bodies[i].mass
            * (bodies[i].vx * bodies[i].vx + bodies[i].vy * bodies[i].vy + bodies[i].vz * bodies[i].vz);
    }
    for (let i = 0; i < bodyCount; i++) {
        for (let j = i + 1; j < bodyCount; j++) {
            const dx = bodies[i].x - bodies[j].x;
            const dy = bodies[i].y - bodies[j].y;
            const dz = bodies[i].z - bodies[j].z;
            energy -= bodies[i].mass * bodies[j].mass / Math.sqrt(dx * dx + dy * dy + dz * dz);
        }
    }

    return energy;
}
