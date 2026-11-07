on run() -> int {
    let item_count = 10000
    let modulus = 1000000007
    let lcg_modulus = 2147483647
    let lcg_multiplier = 48271

    var values: list[int] = []
    var seed = 12345
    var index = 0
    while index < item_count {
        seed = (seed * lcg_multiplier) % lcg_modulus
        values.push(seed)
        index += 1
    }

    var stack: list[(int, int)] = [(0, values.len() - 1)]
    while stack.len() > 0 {
        let next = stack.pop()
        match next {
            Some(bounds) => {
                let lo = bounds.0
                let hi = bounds.1
                if lo < hi {
                    let pivot_index = (lo + hi) / 2
                    let pivot = values[pivot_index]
                    values[pivot_index] = values[hi]
                    values[hi] = pivot

                    var split = lo
                    var scan = lo
                    while scan < hi {
                        if values[scan] < pivot {
                            let swap_value = values[scan]
                            values[scan] = values[split]
                            values[split] = swap_value
                            split += 1
                        }
                        scan += 1
                    }

                    let final_swap = values[split]
                    values[split] = values[hi]
                    values[hi] = final_swap

                    stack.push((lo, split - 1))
                    stack.push((split + 1, hi))
                }
            }
            None => (),
        }
    }

    var checksum = 0
    var checksum_index = 0
    while checksum_index < values.len() {
        checksum = (checksum + values[checksum_index] * (checksum_index + 1)) % modulus
        checksum_index += 1
    }
    checksum
}
