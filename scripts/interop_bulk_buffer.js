function bench_bulk_f64_in_buffer(values) {
    let sum = 0;
    for (let index = 0; index < values.length; index++) {
        sum += values[index];
    }
    return sum;
}

function bench_bulk_u8_in_buffer(values) {
    let sum = 0;
    for (let index = 0; index < values.length; index++) {
        sum += values[index];
    }
    return sum;
}

function make_f64_buffer(size) {
    const values = new Float64Array(size);
    values.fill(1);
    return values;
}

function make_u8_buffer(size) {
    const values = new Uint8Array(size);
    values.fill(1);
    return values;
}

const f64_buffer_1k = make_f64_buffer(1000);
const f64_buffer_100k = make_f64_buffer(100000);
const f64_buffer_1m = make_f64_buffer(1000000);
const u8_buffer_1k = make_u8_buffer(1000);
const u8_buffer_100k = make_u8_buffer(100000);
const u8_buffer_1m = make_u8_buffer(1000000);

function bench_bulk_f64_out_buffer_1k() {
    return interop_sum_f64_buffer(f64_buffer_1k);
}

function bench_bulk_f64_out_buffer_100k() {
    return interop_sum_f64_buffer(f64_buffer_100k);
}

function bench_bulk_f64_out_buffer_1m() {
    return interop_sum_f64_buffer(f64_buffer_1m);
}

function bench_bulk_u8_out_buffer_1k() {
    return interop_sum_u8_buffer(u8_buffer_1k);
}

function bench_bulk_u8_out_buffer_100k() {
    return interop_sum_u8_buffer(u8_buffer_100k);
}

function bench_bulk_u8_out_buffer_1m() {
    return interop_sum_u8_buffer(u8_buffer_1m);
}
