const count = 10000;
const passes = 50;
const modulus = 1000000007;

function bench() {
    const values = [];
    for (let index = 0; index < count; index++) {
        values.push(index);
    }

    let checksum = 0;
    for (let pass = 0; pass < passes; pass++) {
        const mapped = values.map((value) => value * 2);
        const filtered = mapped.filter((value) => value % 3 === 0);
        const sum = filtered.reduce((accumulator, value) => accumulator + value, 0);
        checksum = (checksum + sum) % modulus;
    }
    return checksum;
}
