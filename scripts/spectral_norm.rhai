let n = 300;
let iterations = 10;

fn eval_a(i, j) {
    let base = (i + j) * (i + j + 1) / 2 + i + 1;
    1.0 / base
}

let u = [];
let v = [];
for i in 0..n {
    u.push(1.0);
    v.push(0.0);
}

for _iteration in 0..iterations {
    for i in 0..n {
        let sum = 0.0;
        for j in 0..n {
            sum += eval_a(i, j) * u[j];
        }
        v[i] = sum;
    }

    for i in 0..n {
        u[i] = 0.0;
    }
    for i in 0..n {
        let value = v[i];
        for j in 0..n {
            u[j] += eval_a(i, j) * value;
        }
    }
}

for i in 0..n {
    let sum = 0.0;
    for j in 0..n {
        sum += eval_a(i, j) * u[j];
    }
    v[i] = sum;
}

let v_bv = 0.0;
let vv = 0.0;
for i in 0..n {
    v_bv += u[i] * v[i];
    vv += v[i] * v[i];
}

(v_bv / vv).sqrt()
