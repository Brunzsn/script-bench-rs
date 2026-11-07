let depth = 14;
let iterations = 20;

fn build(level) {
    if level == 0 {
        ()
    } else {
        [build(level - 1), build(level - 1)]
    }
}

fn count(node) {
    if node == () {
        1
    } else {
        1 + count(node[0]) + count(node[1])
    }
}

let total = 0;
for _iteration in 0..iterations {
    total += count(build(depth));
}
total
