fn bench_bulk_f64_in_list(values) {
    let sum = 0.0;
    for value in values {
        sum += value;
    }
    sum
}

fn bench_bulk_u8_in_list(values) {
    let sum = 0.0;
    for value in values {
        sum += value;
    }
    sum
}

fn bench_bulk_f64_out_list_1k() {
    let values = [];
    for index in 0..1000 {
        values.push(1.0);
    }
    interop_sum_f64_list(values)
}

fn bench_bulk_f64_out_list_100k() {
    let values = [];
    for index in 0..100000 {
        values.push(1.0);
    }
    interop_sum_f64_list(values)
}

fn bench_bulk_f64_out_list_1m() {
    let values = [];
    for index in 0..1000000 {
        values.push(1.0);
    }
    interop_sum_f64_list(values)
}

fn bench_bulk_u8_out_list_1k() {
    let values = [];
    for index in 0..1000 {
        values.push(1);
    }
    interop_sum_u8_list(values)
}

fn bench_bulk_u8_out_list_100k() {
    let values = [];
    for index in 0..100000 {
        values.push(1);
    }
    interop_sum_u8_list(values)
}

fn bench_bulk_u8_out_list_1m() {
    let values = [];
    for index in 0..1000000 {
        values.push(1);
    }
    interop_sum_u8_list(values)
}
