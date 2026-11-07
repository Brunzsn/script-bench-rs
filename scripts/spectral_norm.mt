fn eval_a(i: int, j: int) -> float {
    let i_float = float(i)
    let j_float = float(j)
    1.0 / ((i_float + j_float) * (i_float + j_float + 1.0) / 2.0 + i_float + 1.0)
}

on run() -> float {
    let n = 300
    let iterations = 10

    var u: list[float] = []
    var v: list[float] = []
    var index = 0
    while index < n {
        u.push(1.0)
        v.push(0.0)
        index += 1
    }

    var iteration = 0
    while iteration < iterations {
        var i = 0
        while i < n {
            var sum = 0.0
            var j = 0
            while j < n {
                sum += eval_a(i, j) * u[j]
                j += 1
            }
            v[i] = sum
            i += 1
        }

        i = 0
        while i < n {
            u[i] = 0.0
            i += 1
        }
        i = 0
        while i < n {
            let value = v[i]
            var j = 0
            while j < n {
                u[j] += eval_a(i, j) * value
                j += 1
            }
            i += 1
        }
        iteration += 1
    }

    var i = 0
    while i < n {
        var sum = 0.0
        var j = 0
        while j < n {
            sum += eval_a(i, j) * u[j]
            j += 1
        }
        v[i] = sum
        i += 1
    }

    var v_bv = 0.0
    var vv = 0.0
    index = 0
    while index < n {
        v_bv += u[index] * v[index]
        vv += v[index] * v[index]
        index += 1
    }

    (v_bv / vv).sqrt()
}
