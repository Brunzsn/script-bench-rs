use std::hint::black_box;
use std::f64::consts::PI;

use anyhow::Result;

use crate::harness::{ensure_close, expect_float, Case, ResultKind, Scripts, Value};

const BODY_COUNT: usize = 5;
const STEP_COUNT: i64 = 100_000;
const DAYS_PER_YEAR: f64 = 365.24;
const SOLAR_MASS: f64 = 4.0 * PI * PI;
const DT: f64 = 0.01;

#[derive(Clone, Copy)]
struct Body {
    x: f64,
    y: f64,
    z: f64,
    vx: f64,
    vy: f64,
    vz: f64,
    mass: f64,
}

pub const CASE: Case = Case {
    name: "N-body",
    scripts: Scripts {
        js: Some("scripts/nbody.js"),
        koto: Some("scripts/nbody.koto"),
        lua: Some("scripts/nbody.lua"),
        martite_script: Some("scripts/nbody.mt"),
        martite_interface: Some("scripts/nbody.mti"),
        rhai: Some("scripts/nbody.rhai"),
        roto: None,
        wasm_source: None,
        wasm_module: None,
    },
    result: ResultKind::Float,
    native,
    validate,
};

fn initial_bodies() -> [Body; BODY_COUNT] {
    [
        Body {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
            mass: SOLAR_MASS,
        },
        Body {
            x: 4.84143144246472090e+00,
            y: -1.16032004402742839e+00,
            z: -1.03622044471123109e-01,
            vx: 1.66007664274403694e-03 * DAYS_PER_YEAR,
            vy: 7.69901118419740425e-03 * DAYS_PER_YEAR,
            vz: -6.90460016972063023e-05 * DAYS_PER_YEAR,
            mass: 9.54791938424326609e-04 * SOLAR_MASS,
        },
        Body {
            x: 8.34336671824457987e+00,
            y: 4.12479856412430479e+00,
            z: -4.03523417114321381e-01,
            vx: -2.76742510726862411e-03 * DAYS_PER_YEAR,
            vy: 4.99852801234917238e-03 * DAYS_PER_YEAR,
            vz: 2.30417297573763929e-05 * DAYS_PER_YEAR,
            mass: 2.85885980666130812e-04 * SOLAR_MASS,
        },
        Body {
            x: 1.28943695621391310e+01,
            y: -1.51111514016986312e+01,
            z: -2.23307578892655734e-01,
            vx: 2.96460137564761618e-03 * DAYS_PER_YEAR,
            vy: 2.37847173959480950e-03 * DAYS_PER_YEAR,
            vz: -2.96589568540237556e-05 * DAYS_PER_YEAR,
            mass: 4.36624404335156298e-05 * SOLAR_MASS,
        },
        Body {
            x: 1.53796971148509165e+01,
            y: -2.59193146099879641e+01,
            z: 1.79258772950371181e-01,
            vx: 2.68067772490389322e-03 * DAYS_PER_YEAR,
            vy: 1.62824170038242295e-03 * DAYS_PER_YEAR,
            vz: -9.51592254519715870e-05 * DAYS_PER_YEAR,
            mass: 5.15138902046611451e-05 * SOLAR_MASS,
        },
    ]
}

fn advance(bodies: &mut [Body; BODY_COUNT], dt: f64) {
    for i in 0..BODY_COUNT {
        for j in (i + 1)..BODY_COUNT {
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

    for body in bodies.iter_mut() {
        body.x += dt * body.vx;
        body.y += dt * body.vy;
        body.z += dt * body.vz;
    }
}

fn energy(bodies: &[Body; BODY_COUNT]) -> f64 {
    let mut total = 0.0f64;
    for body in bodies {
        total += 0.5
            * body.mass
            * (body.vx * body.vx + body.vy * body.vy + body.vz * body.vz);
    }
    for i in 0..BODY_COUNT {
        for j in (i + 1)..BODY_COUNT {
            let dx = bodies[i].x - bodies[j].x;
            let dy = bodies[i].y - bodies[j].y;
            let dz = bodies[i].z - bodies[j].z;
            total -= bodies[i].mass * bodies[j].mass / (dx * dx + dy * dy + dz * dz).sqrt();
        }
    }
    total
}

fn reference() -> f64 {
    let mut bodies = initial_bodies();
    for _ in 0..STEP_COUNT {
        advance(&mut bodies, DT);
    }
    energy(&bodies)
}

fn native() {
    black_box(reference());
}

fn validate(value: &Value) -> Result<()> {
    let actual = expect_float(value)?;
    ensure_close(actual, reference(), 1e-6)
}
