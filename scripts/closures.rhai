let count = 10000;
let passes = 50;
let modulus = 1000000007;

let values = [];
for index in 0..count {
    values.push(index);
}

let checksum = 0;
for _pass in 0..passes {
    let mapped = values.map(|value| value * 2);
    let filtered = mapped.filter(|value| value % 3 == 0);
    let sum = filtered.reduce(|accumulator, value| accumulator + value, 0);
    checksum = (checksum + sum) % modulus;
}
checksum
