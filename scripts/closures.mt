// allow(L001)
use bench.data

on run() -> int {
    let count = 10000
    let passes = 50
    let modulus = 1000000007

    var values: list[int] = []
    var index = 0
    while index < count {
        values.push(index)
        index += 1
    }

    var checksum = 0
    var pass = 0
    while pass < passes {
        let mapped = values.map(fn(value) => value * 2)
        let filtered = mapped.filter(fn(value) => value % 3 == 0)
        let sum = filtered.fold(0, fn(accumulator, value) => accumulator + value)
        checksum = (checksum + sum) % modulus
        pass += 1
    }
    checksum
}
