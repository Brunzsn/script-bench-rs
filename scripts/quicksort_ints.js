const itemCount = 10000;
const modulus = 1000000007;
const lcgModulus = 2147483647;
const lcgMultiplier = 48271;

function partition(values, lo, hi) {
    const pivotIndex = Math.floor((lo + hi) / 2);
    let temporary = values[pivotIndex];
    values[pivotIndex] = values[hi];
    values[hi] = temporary;
    const pivot = values[hi];
    let split = lo;
    for (let index = lo; index < hi; index++) {
        if (values[index] < pivot) {
            temporary = values[index];
            values[index] = values[split];
            values[split] = temporary;
            split++;
        }
    }
    temporary = values[split];
    values[split] = values[hi];
    values[hi] = temporary;
    return split;
}

function quicksort(values, lo, hi) {
    while (lo < hi) {
        const pivot = partition(values, lo, hi);
        if (pivot > lo) {
            quicksort(values, lo, pivot - 1);
        }
        lo = pivot + 1;
    }
}

function bench() {
    const values = [];
    let seed = 12345;
    for (let index = 0; index < itemCount; index++) {
        seed = (seed * lcgMultiplier) % lcgModulus;
        values.push(seed);
    }

    quicksort(values, 0, values.length - 1);

    let checksum = 0;
    for (let index = 0; index < values.length; index++) {
        checksum = (checksum + values[index] * (index + 1)) % modulus;
    }
    return checksum;
}
