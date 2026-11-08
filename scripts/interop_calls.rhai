fn bench_call_empty() {
    let sum = 0;
    for index in 0..100000 {
        sum += interop_ping();
    }
    sum
}

fn bench_call_one_i64() {
    let sum = 0;
    for index in 0..100000 {
        sum += interop_id_i64(7);
    }
    sum
}

fn bench_call_three_i64() {
    let sum = 0;
    for index in 0..100000 {
        sum += interop_sum_i64_3(1, 2, 3);
    }
    sum
}

fn bench_call_five_i64() {
    let sum = 0;
    for index in 0..100000 {
        sum += interop_sum_i64_5(1, 2, 3, 4, 5);
    }
    sum
}

fn bench_call_f64() {
    let sum = 0.0;
    for index in 0..100000 {
        sum += interop_id_f64(1.5);
    }
    sum
}

fn bench_call_string_in() {
    let sum = 0;
    for index in 0..100000 {
        sum += interop_strlen("interop");
    }
    sum
}

fn bench_call_string_out() {
    let sum = 0;
    for index in 0..100000 {
        sum += interop_string_out().len;
    }
    sum
}
