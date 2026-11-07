let item_count = 10000;
let modulus = 1000000007;
let lcg_modulus = 2147483647;
let lcg_multiplier = 48271;

fn swap(i, j) {
    let temporary = this[i];
    this[i] = this[j];
    this[j] = temporary;
}

fn partition(lo, hi) {
    let pivot_index = (lo + hi) / 2;
    this.swap(pivot_index, hi);
    let pivot = this[hi];
    let split = lo;
    for index in lo..hi {
        if this[index] < pivot {
            this.swap(index, split);
            split += 1;
        }
    }
    this.swap(split, hi);
    split
}

fn quicksort(lo, hi) {
    while lo < hi {
        let pivot = this.partition(lo, hi);
        if pivot > lo {
            this.quicksort(lo, pivot - 1);
        }
        lo = pivot + 1;
    }
}

let values = [];
let seed = 12345;
for _item in 0..item_count {
    seed = (seed * lcg_multiplier) % lcg_modulus;
    values.push(seed);
}

values.quicksort(0, values.len - 1);

let checksum = 0;
for index in 0..values.len {
    checksum = (checksum + values[index] * (index + 1)) % modulus;
}
checksum
