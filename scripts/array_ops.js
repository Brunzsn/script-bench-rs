const count = 100000;
const modulus = 1000000007;

function bench() {
    const values = [];
    for (let index = 0; index < count; index++) {
        values.push(index);
    }

    let indexedSum = 0;
    for (let index = 0; index < count; index++) {
        indexedSum += values[index];
    }

    let iteratedSum = 0;
    for (const value of values) {
        iteratedSum += value;
    }

    return (indexedSum + iteratedSum) % modulus;
}
