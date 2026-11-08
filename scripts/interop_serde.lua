function bench_serde_to_script(value)
    return value.id
        + #value.name
        + value.values[1]
        + value.values[2]
        + value.values[3]
        + value.child.id
end

function bench_serde_from_script()
    return {
        id = 7,
        name = "interop",
        values = { 1.0, 2.0, 3.0 },
        child = { id = 9 },
    }
end
