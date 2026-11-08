function sum_f64_list(values) {
    let sum = 0;
    for (let index = 0; index < values.length; index++) {
        sum += values[index];
    }
    return sum;
}

function bench_bulk_f64_in_list(values) {
    return sum_f64_list(values);
}

function bench_bulk_u8_in_list(values) {
    return sum_f64_list(values);
}

function make_unit_list(size, value) {
    const values = new Array(size);
    for (let index = 0; index < size; index++) {
        values[index] = value;
    }
    return values;
}

const f64_list_1k = make_unit_list(1000, 1.0);
const f64_list_100k = make_unit_list(100000, 1.0);
const f64_list_1m = make_unit_list(1000000, 1.0);
const u8_list_1k = make_unit_list(1000, 1);
const u8_list_100k = make_unit_list(100000, 1);
const u8_list_1m = make_unit_list(1000000, 1);

function bench_bulk_f64_out_list_1k() {
    return interop_sum_f64_list(f64_list_1k);
}

function bench_bulk_f64_out_list_100k() {
    return interop_sum_f64_list(f64_list_100k);
}

function bench_bulk_f64_out_list_1m() {
    return interop_sum_f64_list(f64_list_1m);
}

function bench_bulk_u8_out_list_1k() {
    return interop_sum_u8_list(u8_list_1k);
}

function bench_bulk_u8_out_list_100k() {
    return interop_sum_u8_list(u8_list_100k);
}

function bench_bulk_u8_out_list_1m() {
    return interop_sum_u8_list(u8_list_1m);
}
