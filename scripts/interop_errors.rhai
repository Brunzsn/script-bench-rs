fn bench_error_caught() {
    let count = 0;
    for index in 0..10000 {
        try {
            interop_fail();
        } catch (error) {
            count += 1;
        }
    }
    count
}

fn bench_error_propagates() {
    interop_fail();
}
