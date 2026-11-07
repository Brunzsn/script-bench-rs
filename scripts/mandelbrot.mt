// allow(L001)
use bench.data

// allow(L017) -- the pixel loops stay inline to measure the arithmetic directly.
on run() -> float {
    let size = 400
    let size_float = float(size)
    let max_iterations = 50
    var checksum = 0.0
    var y = 0

    while y < size {
        let ci = 2.0 * float(y) / size_float - 1.0
        var x = 0
        while x < size {
            let cr = 2.0 * float(x) / size_float - 1.5
            var zr = 0.0
            var zi = 0.0
            var iteration = 0
            while iteration < max_iterations {
                if zr * zr + zi * zi > 4.0 { break }
                let next_zr = zr * zr - zi * zi + cr
                zi = 2.0 * zr * zi + ci
                zr = next_zr
                iteration += 1
            }
            checksum += float(iteration)
            x += 1
        }
        y += 1
    }

    checksum
}
