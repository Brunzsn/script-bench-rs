function bench() {
    let value = 1.0;
    for (let index = 0; index < 5000000; index++) {
        value = value * 0.999999 + 0.000001;
    }
    return value;
}
