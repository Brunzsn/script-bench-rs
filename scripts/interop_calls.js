function bench_call_empty() {
    let sum = 0;
    for (let index = 0; index < 100000; index++) {
        sum += interop_ping();
    }
    return sum;
}

function bench_call_one_i64() {
    let sum = 0;
    for (let index = 0; index < 100000; index++) {
        sum += interop_id_i64(7);
    }
    return sum;
}

function bench_call_three_i64() {
    let sum = 0;
    for (let index = 0; index < 100000; index++) {
        sum += interop_sum_i64_3(1, 2, 3);
    }
    return sum;
}

function bench_call_five_i64() {
    let sum = 0;
    for (let index = 0; index < 100000; index++) {
        sum += interop_sum_i64_5(1, 2, 3, 4, 5);
    }
    return sum;
}

function bench_call_f64() {
    let sum = 0;
    for (let index = 0; index < 100000; index++) {
        sum += interop_id_f64(1.5);
    }
    return sum;
}

function bench_call_string_in() {
    let sum = 0;
    for (let index = 0; index < 100000; index++) {
        sum += interop_strlen("interop");
    }
    return sum;
}

function bench_call_string_out() {
    let sum = 0;
    for (let index = 0; index < 100000; index++) {
        sum += interop_string_out().length;
    }
    return sum;
}
