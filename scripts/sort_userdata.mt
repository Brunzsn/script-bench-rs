use bench.data
use bench.data.RustData

const CHARSET: list[str] = [
    "0",
    "1",
    "2",
    "3",
    "4",
    "5",
    "6",
    "7",
    "8",
    "9",
    "a",
    "b",
    "c",
    "d",
    "e",
    "f",
]

fn generate_string(len: int) -> str {
    var chars: list[str] = []
    var index = 0
    while index < len {
        chars.push(CHARSET[data.rand(CHARSET.len())])
        index += 1
    }
    chars.join("")
}

// allow(L017) -- the sort stays inline so the large list is never passed by value.
on run() -> list[RustData] {
    var values: list[RustData] = []
    var count = 0
    while count < 10000 {
        values.push(data.rust_data_new(generate_string(8 + data.rand(16))))
        count += 1
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
                        if data.rust_data_lt(values[scan], pivot) {
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

    values
}
