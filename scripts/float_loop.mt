// allow(L001)
use bench.data

on run() -> float {
    var value = 1.0
    var index = 0
    while index < 5000000 {
        value = value * 0.999999 + 0.000001
        index += 1
    }
    value
}
