use script_bench::cases::CASES;
use script_bench::harness::Engine;

/// Sets up and validates an engine outside the measured instruction path.
pub fn setup<E: Engine>() -> E {
    let case = &CASES[0];
    let mut engine = E::setup(case).expect("engine setup failed");
    script_bench::rng::reset();
    engine.validate(case).expect("engine validation failed");
    engine
}

/// Runs the timed path once with a fresh deterministic RNG state.
pub fn run<E: Engine>(engine: &mut E) {
    script_bench::rng::reset();
    engine.run().expect("benchmark run failed");
}
