const itemCount = 20000;
const modulus = 1000000007;
const lcgModulus = 2147483647;
const lcgMultiplier = 48271;

function bench() {
    const values = [];
    let seed = 54321;
    for (let index = 0; index < itemCount; index++) {
        seed = (seed * lcgMultiplier) % lcgModulus;
        values.push(seed);
    }

    values.sort((left, right) => right - left);

    let checksum = 0;
    for (let index = 0; index < values.length; index++) {
        checksum = (checksum + values[index] * (index + 1)) % modulus;
    }
    return checksum;
}
