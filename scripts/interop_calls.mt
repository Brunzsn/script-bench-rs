use bench.data

on bench_call_empty() -> int {
    var sum = 0
    var index = 0
    while index < 100000 {
        sum += data.interop_ping()
        index += 1
    }
    sum
}

on bench_call_one_i64() -> int {
    var sum = 0
    var index = 0
    while index < 100000 {
        sum += data.interop_id_i64(7)
        index += 1
    }
    sum
}

on bench_call_three_i64() -> int {
    var sum = 0
    var index = 0
    while index < 100000 {
        sum += data.interop_sum_i64_3(1, 2, 3)
        index += 1
    }
    sum
}

on bench_call_five_i64() -> int {
    var sum = 0
    var index = 0
    while index < 100000 {
        sum += data.interop_sum_i64_5(1, 2, 3, 4, 5)
        index += 1
    }
    sum
}

on bench_call_f64() -> float {
    var sum = 0.0
    var index = 0
    while index < 100000 {
        sum += data.interop_id_f64(1.5)
        index += 1
    }
    sum
}

on bench_call_string_in() -> int {
    var sum = 0
    var index = 0
    while index < 100000 {
        sum += data.interop_strlen("interop")
        index += 1
    }
    sum
}

on bench_call_string_out() -> int {
    var sum = 0
    var index = 0
    while index < 100000 {
        sum += data.interop_string_out().len()
        index += 1
    }
    sum
}
