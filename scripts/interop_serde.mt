use bench.data
use bench.data.Nested
use bench.data.NestedChild

on bench_serde_from_script() -> Nested {
    Nested {
        id: 7,
        name: "interop",
        values: [1.0, 2.0, 3.0],
        child: NestedChild { id: 9 },
    }
}
