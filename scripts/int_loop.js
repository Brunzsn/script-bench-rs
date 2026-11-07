function bench() {
    const modulus = 1000000007;
    let value = 12345;
    for (let index = 0; index < 10000000; index++) {
        value = (value * 31 + index) % modulus;
    }
    return value;
}
