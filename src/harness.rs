use std::path::{Path, PathBuf};

use anyhow::{bail, Context as _, Result};

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

/// How a case's entry point encodes its result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultKind {
    /// A list of `RustData` userdata objects, as returned by the sort case.
    UserDataList,
    /// An integer checksum or count.
    Int,
    /// A floating-point checksum.
    Float,
}

/// Paths to the scripts implementing a case, one per engine.
///
/// Engines that cannot express a case without host calls leave the relevant
/// field as `None`; the case is then skipped for that engine.
#[derive(Clone, Copy)]
pub struct Scripts {
    pub js: Option<&'static str>,
    pub koto: Option<&'static str>,
    pub lua: Option<&'static str>,
    pub martite_script: Option<&'static str>,
    pub martite_interface: Option<&'static str>,
    pub rhai: Option<&'static str>,
    pub roto: Option<&'static str>,
    pub wasm_source: Option<&'static str>,
    pub wasm_module: Option<&'static str>,
}

impl Scripts {
    /// Returns the wasm source and module path when the case has a wasm build.
    pub fn wasm(&self) -> Option<(&'static str, &'static str)> {
        match (self.wasm_source, self.wasm_module) {
            (Some(source), Some(module)) => Some((source, module)),
            _ => None,
        }
    }
}

/// A benchmark case shared by every engine.
pub struct Case {
    /// Criterion benchmark id, also used as the plot title and file name.
    pub name: &'static str,
    /// Script paths for each engine.
    pub scripts: Scripts,
    /// Encoding of the result returned by the script entry point.
    pub result: ResultKind,
    /// Native Rust reference implementation, used for the baseline and the
    /// expected checksum. This is the timed path of the `native` bench target.
    pub native: fn(),
    /// Checks the canonical value returned by the script entry point.
    pub validate: fn(&Value) -> Result<()>,
}

/// An engine adapter that can run benchmark cases.
pub trait Engine: Sized {
    /// Initialise the engine and load `case`'s script, without running it.
    ///
    /// Returns `None` when the engine has no script for the case.
    fn setup(case: &Case) -> Result<Option<Self>>;

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
        let Some(mut engine) =
            E::setup(case).with_context(|| format!("setting up case `{}`", case.name))?
        else {
            continue;
        };
        crate::rng::reset();
        engine
            .validate(case)
            .with_context(|| format!("validating case `{}`", case.name))?;

        bench(case.name, &mut || {
            crate::rng::reset();
            engine
                .run()
                .unwrap_or_else(|error| panic!("case `{}` failed: {error:#}", case.name));
        });
    }

    Ok(())
}

/// Converts a numeric script result into the case's declared result kind.
pub fn number_value(kind: ResultKind, number: f64) -> Result<Value> {
    match kind {
        ResultKind::UserDataList => Ok(Value::Float(number)),
        ResultKind::Int => {
            if !number.is_finite() || number.fract() != 0.0 {
                bail!("expected an integer result, got {number}");
            }
            Ok(Value::Int(number as i64))
        }
        ResultKind::Float => Ok(Value::Float(number)),
    }
}

/// Extracts an integer from a validated value.
pub fn expect_int(value: &Value) -> Result<i64> {
    match value {
        Value::Int(number) => Ok(*number),
        Value::Float(number) if number.is_finite() && number.fract() == 0.0 => Ok(*number as i64),
        other => bail!("expected an integer result, got {other:?}"),
    }
}

/// Extracts a float from a validated value.
pub fn expect_float(value: &Value) -> Result<f64> {
    match value {
        Value::Float(number) => Ok(*number),
        Value::Int(number) => Ok(*number as f64),
        other => bail!("expected a floating-point result, got {other:?}"),
    }
}

/// Checks that `actual` is within a relative tolerance of `expected`.
pub fn ensure_close(actual: f64, expected: f64, tolerance: f64) -> Result<()> {
    let limit = tolerance * expected.abs().max(1.0);
    let difference = (actual - expected).abs();
    if difference > limit {
        bail!("result mismatch: got {actual}, expected {expected} (difference {difference}, limit {limit})");
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
