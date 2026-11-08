use bench.data

on bench_userdata_method() -> int {
    let doc = data.interop_new_doc(7)
    var sum = 0
    var index = 0
    while index < 100000 {
        sum += data.interop_method(doc)
        index += 1
    }
    sum
}

on bench_userdata_ctor_rc() -> int {
    var count = 0
    var index = 0
    while index < 100000 {
        let _doc = data.interop_new_doc(7)
        count += 1
        index += 1
    }
    count
}
