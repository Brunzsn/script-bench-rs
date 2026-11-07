on run() -> int {
    let item_count = 20000
    let modulus = 1000000007
    let lcg_modulus = 2147483647
    let lcg_multiplier = 48271

    var values: list[int] = []
    var seed = 54321
    var index = 0
    while index < item_count {
        seed = (seed * lcg_multiplier) % lcg_modulus
        values.push(seed)
        index += 1
    }

    let sorted = values.sorted_by(fn(value) => -value)

    var checksum = 0
    index = 0
    while index < sorted.len() {
        checksum = (checksum + sorted[index] * (index + 1)) % modulus
        index += 1
    }
    checksum
}
