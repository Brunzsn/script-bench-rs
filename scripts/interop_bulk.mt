use bench.data

struct State {
    f64_1k: list[float],
    f64_100k: list[float],
    f64_1m: list[float],
    u8_1k: list[int],
    u8_100k: list[int],
    u8_1m: list[int],
}

on init() -> State {
    var f64_1k: list[float] = []
    var f64_100k: list[float] = []
    var f64_1m: list[float] = []
    var u8_1k: list[int] = []
    var u8_100k: list[int] = []
    var u8_1m: list[int] = []

    var index = 0
    while index < 1_000 {
        f64_1k.push(1.0)
        u8_1k.push(1)
        index += 1
    }
    index = 0
    while index < 100_000 {
        f64_100k.push(1.0)
        u8_100k.push(1)
        index += 1
    }
    index = 0
    while index < 1_000_000 {
        f64_1m.push(1.0)
        u8_1m.push(1)
        index += 1
    }

    State {
        f64_1k,
        f64_100k,
        f64_1m,
        u8_1k,
        u8_100k,
        u8_1m,
    }
}

on bench_bulk_f64_in_list(values: list[float]) -> float {
    var sum = 0.0
    for value in values {
        sum += value
    }
    sum
}

on bench_bulk_u8_in_list(values: list[int]) -> int {
    var sum = 0
    for value in values {
        sum += value
    }
    sum
}

on bench_bulk_f64_out_list_1k(self) -> float {
    data.interop_sum_f64_list(self.f64_1k)
}

on bench_bulk_f64_out_list_100k(self) -> float {
    data.interop_sum_f64_list(self.f64_100k)
}

on bench_bulk_f64_out_list_1m(self) -> float {
    data.interop_sum_f64_list(self.f64_1m)
}

on bench_bulk_u8_out_list_1k(self) -> int {
    data.interop_sum_int_list(self.u8_1k)
}

on bench_bulk_u8_out_list_100k(self) -> int {
    data.interop_sum_int_list(self.u8_100k)
}

on bench_bulk_u8_out_list_1m(self) -> int {
    data.interop_sum_int_list(self.u8_1m)
}
