use bench.data

on bench_error_caught() -> int {
    var count = 0
    var index = 0
    while index < 10000 {
        match data.interop_fail() {
            Err(_) => count += 1,
            Ok(_) => (),
        }
        index += 1
    }
    count
}

on bench_error_propagates() -> Result[(), str] {
    let _ = data.interop_fail()?
    Ok(())
}
