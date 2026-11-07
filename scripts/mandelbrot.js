function bench() {
    const size = 400;
    const sizeFloat = 400.0;
    const maxIterations = 50.0;
    let checksum = 0.0;

    for (let y = 0; y < size; y++) {
        const ci = 2.0 * y / sizeFloat - 1.0;
        for (let x = 0; x < size; x++) {
            const cr = 2.0 * x / sizeFloat - 1.5;
            let zr = 0.0;
            let zi = 0.0;
            let iteration = 0.0;
            while (iteration < maxIterations && zr * zr + zi * zi <= 4.0) {
                const nextZr = zr * zr - zi * zi + cr;
                zi = 2.0 * zr * zi + ci;
                zr = nextZr;
                iteration += 1.0;
            }
            checksum += iteration;
        }
    }

    return checksum;
}
