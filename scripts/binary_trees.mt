enum Tree { Leaf, Node(Tree, Tree) }

fn build(depth: int) -> Tree {
    if depth == 0 {
        Tree.Leaf
    } else {
        Tree.Node(build(depth - 1), build(depth - 1))
    }
}

fn count(tree: Tree) -> int {
    match tree {
        Leaf => 1,
        Node(left, right) => 1 + count(left) + count(right),
    }
}

on run() -> int {
    let depth = 14
    let iterations = 20
    var total = 0
    var iteration = 0
    while iteration < iterations {
        total += count(build(depth))
        iteration += 1
    }
    total
}
