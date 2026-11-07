const depth = 14;
const iterations = 20;

function build(level) {
    if (level === 0) {
        return null;
    }
    return { left: build(level - 1), right: build(level - 1) };
}

function count(node) {
    if (node === null) {
        return 1;
    }
    return 1 + count(node.left) + count(node.right);
}

function bench() {
    let total = 0;
    for (let iteration = 0; iteration < iterations; iteration++) {
        total += count(build(depth));
    }
    return total;
}
