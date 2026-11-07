on run() -> int {
    let count = 20000
    let modulus = 1000000007

    var table: map[str, int] = {}
    var index = 0
    while index < count {
        table["key{index}"] = index * 3
        index += 1
    }

    var looked_up = 0
    index = 0
    while index < count {
        looked_up += table["key{index}"]
        index += 1
    }

    var iterated = 0
    for (_, value) in table {
        iterated += value
    }

    (looked_up + iterated + table.len()) % modulus
}
