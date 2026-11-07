let count = 100000;
let modulus = 1000000007;

let values = [];
for index in 0..count {
    values.push(index);
}

let indexed_sum = 0;
for index in 0..count {
    indexed_sum += values[index];
}

let iterated_sum = 0;
for value in values {
    iterated_sum += value;
}

(indexed_sum + iterated_sum) % modulus
