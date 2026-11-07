// allow(L001)
use bench.data

on run() -> int {
    let count = 100000
    let modulus = 1000000007

    var values: list[int] = []
    var index = 0
    while index < count {
        values.push(index)
        index += 1
    }

    var indexed_sum = 0
    index = 0
    while index < count {
        indexed_sum += values[index]
        index += 1
    }

    var iterated_sum = 0
    for value in values {
        iterated_sum += value
    }

    (indexed_sum + iterated_sum) % modulus
}
