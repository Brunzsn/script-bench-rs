function bench_error_caught() {
    let count = 0;
    for (let index = 0; index < 10000; index++) {
        try {
            interop_fail();
        } catch (error) {
            count++;
        }
    }
    return count;
}

function bench_error_propagates() {
    interop_fail();
}
