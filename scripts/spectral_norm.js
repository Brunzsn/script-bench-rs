const n = 300;
const iterations = 10;

function evalA(i, j) {
    return 1.0 / ((i + j) * (i + j + 1) / 2.0 + i + 1.0);
}

function bench() {
    const u = [];
    const v = [];
    for (let i = 0; i < n; i++) {
        u.push(1.0);
        v.push(0.0);
    }

    for (let iteration = 0; iteration < iterations; iteration++) {
        for (let i = 0; i < n; i++) {
            let sum = 0.0;
            for (let j = 0; j < n; j++) {
                sum += evalA(i, j) * u[j];
            }
            v[i] = sum;
        }

        for (let i = 0; i < n; i++) {
            u[i] = 0.0;
        }
        for (let i = 0; i < n; i++) {
            const value = v[i];
            for (let j = 0; j < n; j++) {
                u[j] += evalA(i, j) * value;
            }
        }
    }

    for (let i = 0; i < n; i++) {
        let sum = 0.0;
        for (let j = 0; j < n; j++) {
            sum += evalA(i, j) * u[j];
        }
        v[i] = sum;
    }

    let vBv = 0.0;
    let vv = 0.0;
    for (let i = 0; i < n; i++) {
        vBv += u[i] * v[i];
        vv += v[i] * v[i];
    }
    return Math.sqrt(vBv / vv);
}
