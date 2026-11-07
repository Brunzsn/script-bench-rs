// allow(L001)
use bench.data

// allow(L017) -- the allocation loop stays inline so nodes are pushed directly.
on run() -> int {
    let depth = 14
    let iterations = 20
    var total = 0
    var iteration = 0

    while iteration < iterations {
        // Build a full binary tree as a flat list of node depths. The
        // reference implementation uses a recursive enum, but Martite's
        // recursive type layout is not usable here, so this keeps the same
        // allocation count without a recursive value.
        var nodes: list[int] = []
        var stack: list[int] = [depth]
        while stack.len() > 0 {
            let next = stack.pop()
            match next {
                Some(level) => {
                    nodes.push(level)
                    if level > 0 {
                        stack.push(level - 1)
                        stack.push(level - 1)
                    }
                }
                None => (),
            }
        }
        total += nodes.len()
        iteration += 1
    }

    total
}
