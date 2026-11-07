let item_count = 20000;
let modulus = 1000000007;
let lcg_modulus = 2147483647;
let lcg_multiplier = 48271;

let values = [];
let seed = 54321;
for _item in 0..item_count {
    seed = (seed * lcg_multiplier) % lcg_modulus;
    values.push(seed);
}

values.sort(|left, right| right - left);

let checksum = 0;
for index in 0..values.len {
    checksum = (checksum + values[index] * (index + 1)) % modulus;
}
checksum
