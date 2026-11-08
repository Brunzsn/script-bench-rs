function bench_userdata_method() {
    const object = new InteropData(7);
    let sum = 0;
    for (let index = 0; index < 100000; index++) {
        sum += object.get();
    }
    return sum;
}

function bench_userdata_field_get() {
    const object = new InteropData(7);
    let sum = 0;
    for (let index = 0; index < 100000; index++) {
        sum += object.value;
    }
    return sum;
}

function bench_userdata_field_set() {
    const object = new InteropData(0);
    for (let index = 0; index < 100000; index++) {
        object.value = index;
    }
    return object.value;
}

function bench_userdata_ctor_copy() {
    let count = 0;
    for (let index = 0; index < 100000; index++) {
        new CopyData(7);
        count++;
    }
    return count;
}

function bench_userdata_ctor_rc() {
    let count = 0;
    for (let index = 0; index < 100000; index++) {
        new RcData("interop");
        count++;
    }
    return count;
}
