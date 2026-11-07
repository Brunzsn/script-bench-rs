use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};

/// Engine-neutral representation of a script result, used for validation.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Vec<Value>),
    /// An opaque host object (e.g. `RustData`), identified by its token.
    Object(String),
}

/// Paths to the scripts implementing a case, one per engine.
pub struct Scripts {
    pub js: &'static str,
    pub koto: &'static str,
    pub lua: &'static str,
    pub martite_script: &'static str,
    pub martite_interface: &'static str,
    pub rhai: &'static str,
    pub roto: &'static str,
    pub wasm_source: &'static str,
    pub wasm_module: &'static str,
}

/// A benchmark case shared by every engine.
pub struct Case {
    /// Criterion benchmark id, also used as the plot title and file name.
    pub name: &'static str,
    /// Script paths for each engine.
    pub scripts: Scripts,
    /// Checks the canonical value returned by the script entry point.
    pub validate: fn(&Value) -> Result<()>,
}

/// An engine adapter that can run benchmark cases.
pub trait Engine: Sized {
    /// Initialise the engine and load `case`'s script, without running it.
    fn setup(case: &Case) -> Result<Self>;

    /// Execute the case entry point once. This is the timed path.
    fn run(&mut self) -> Result<()>;

    /// Execute the case entry point once and decode the result for validation.
    ///
    /// This is deliberately separate from [`Engine::run`] so that result
    /// decoding stays out of the measured code path.
    fn run_checked(&mut self) -> Result<Value>;

    /// Run the case once and validate its result.
    fn validate(&mut self, case: &Case) -> Result<()> {
        let value = self.run_checked()?;
        (case.validate)(&value)
    }
}

/// Run every registered case on `E`, wiring each case into `bench`.
pub fn run_cases<E: Engine>(
    mut bench: impl FnMut(&'static str, &mut dyn FnMut()),
) -> Result<()> {
    for case in crate::cases::CASES {
        let mut engine =
            E::setup(case).with_context(|| format!("setting up case `{}`", case.name))?;
        engine
            .validate(case)
            .with_context(|| format!("validating case `{}`", case.name))?;

        bench(case.name, &mut || {
            engine
                .run()
                .unwrap_or_else(|error| panic!("case `{}` failed: {error:#}", case.name));
        });
    }

    Ok(())
}

/// Resolve a repository-relative script path.
pub fn script_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

/// Read a UTF-8 script from the repository.
pub fn read_script(path: &str) -> Result<String> {
    let path = script_path(path);
    std::fs::read_to_string(&path).with_context(|| format!("failed to read `{}`", path.display()))
}

/// Read a binary script (e.g. a compiled wasm module) from the repository.
pub fn read_script_bytes(path: &str) -> Result<Vec<u8>> {
    let path = script_path(path);
    std::fs::read(&path).with_context(|| format!("failed to read `{}`", path.display()))
}
