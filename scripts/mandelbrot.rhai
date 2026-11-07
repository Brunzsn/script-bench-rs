let size = 400;
let size_float = 400.0;
let max_iterations = 50.0;
let checksum = 0.0;

for y in 0..size {
    let ci = 2.0 * y / size_float - 1.0;
    for x in 0..size {
        let cr = 2.0 * x / size_float - 1.5;
        let zr = 0.0;
        let zi = 0.0;
        let iteration = 0.0;
        while iteration < max_iterations && zr * zr + zi * zi <= 4.0 {
            let next_zr = zr * zr - zi * zi + cr;
            zi = 2.0 * zr * zi + ci;
            zr = next_zr;
            iteration += 1.0;
        }
        checksum += iteration;
    }
}

checksum
