fn bench_userdata_method() {
    let object = InteropData_new(7);
    let sum = 0;
    for index in 0..100000 {
        sum += object.get();
    }
    sum
}

fn bench_userdata_field_get() {
    let object = InteropData_new(7);
    let sum = 0;
    for index in 0..100000 {
        sum += object.value;
    }
    sum
}

fn bench_userdata_field_set() {
    let object = InteropData_new(0);
    for index in 0..100000 {
        object.value = index;
    }
    object.value
}

fn bench_userdata_ctor_copy() {
    let count = 0;
    for index in 0..100000 {
        let object = CopyData_new(7);
        count += 1;
    }
    count
}

fn bench_userdata_ctor_rc() {
    let count = 0;
    for index in 0..100000 {
        let object = RcData_new("interop");
        count += 1;
    }
    count
}
