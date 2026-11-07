// allow(L001)
use bench.data

on run() -> int {
    let modulus = 1000000007
    var value = 12345
    var index = 0
    while index < 10000000 {
        value = (value * 31 + index) % modulus
        index += 1
    }
    value
}
