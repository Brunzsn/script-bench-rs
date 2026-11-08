//! Interop-overhead benchmark cases.
//!
//! These cases measure crossing the Rust/script boundary rather than raw
//! interpreter throughput. They use a separate registry and runner trait from
//! [`crate::harness::Case`] because the timed path is often driven from Rust
//! (calling a script function, passing a buffer, observing an error, ...).

use anyhow::{Context as _, Result};

use crate::harness::{ResultKind, Scripts, Value};

pub mod bulk;
pub mod call_in;
pub mod callbacks;
pub mod calls;
pub mod errors;
pub mod host;
pub mod serde;
pub mod userdata;

/// A bulk-data element type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BulkValue {
    F64,
    U8,
}

/// Which direction bulk data travels across the boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Rust creates the value and passes it to the script.
    In,
    /// The script creates the value and passes it to Rust.
    Out,
}

/// How bulk data is represented at the boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BulkMode {
    /// Engine-native list/table/array, converted element by element.
    List,
    /// Typed array or buffer, converted in bulk (and zero-copy for reading
    /// where the engine exposes a view).
    Buffer,
}

/// Bulk-data case parameters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BulkOp {
    pub value: BulkValue,
    pub direction: Direction,
    pub mode: BulkMode,
    pub size: usize,
}

/// Userdata operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UserdataOp {
    /// Call a host method/function on a host object.
    Method,
    /// Read a property/field.
    FieldGet,
    /// Write a property/field.
    FieldSet,
    /// Construct a copy-like host value.
    ConstructorCopy,
    /// Construct an `Rc`/handle-like host value.
    ConstructorRc,
}

/// Serde-style nested conversion direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SerdeOp {
    ToScript,
    FromScript,
}

/// Error-path operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorOp {
    /// The script calls a failing host function and handles the error.
    CaughtInScript,
    /// The script returns/throws an error that Rust observes.
    PropagatesToHost,
}

/// The operation an engine should perform for an interop case.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteropOp {
    /// The script entry point loops over host calls on its own.
    Call,
    /// Rust calls the script entry point repeatedly.
    CallIn { iterations: usize },
    /// The script entry point exercises host userdata.
    Userdata(UserdataOp),
    /// Rust invokes a persistent script closure/hook repeatedly.
    Callback { iterations: usize },
    /// Bulk data crosses the boundary.
    Bulk(BulkOp),
    /// Nested data is converted between Rust and script values.
    Serde(SerdeOp),
    /// An error crosses the boundary.
    Error(ErrorOp),
}

/// A benchmark case that measures interop overhead.
pub struct InteropCase {
    /// Criterion benchmark id, plot title, and output file name.
    pub name: &'static str,
    /// How the case is driven.
    pub op: InteropOp,
    /// Script function, export, or handler that performs the operation.
    pub entry: &'static str,
    /// Script paths per engine. `None` means the engine skips the case.
    pub scripts: Scripts,
    /// Encoding of the value returned by the entry point.
    pub result: ResultKind,
    /// Native Rust reference implementation for the baseline.
    pub native: fn(),
    /// Checks the canonical value returned by the entry point.
    pub validate: fn(&Value) -> Result<()>,
}

/// A runner dedicated to one engine's interop cases.
pub trait InteropEngine: Sized {
    /// Initialise the engine and load `case`'s script without running it.
    ///
    /// Returns `None` when the engine has no script for the case or cannot
    /// express the requested operation.
    fn setup(case: &InteropCase) -> Result<Option<Self>>;

    /// Execute the case entry point once. This is the timed path.
    fn run(&mut self) -> Result<()>;

    /// Execute the case entry point once and decode the result for validation.
    fn run_checked(&mut self) -> Result<Value>;

    /// Run the case once and validate its result.
    fn validate(&mut self, case: &InteropCase) -> Result<()> {
        let value = self.run_checked()?;
        (case.validate)(&value)
    }
}

/// Run every registered interop case on `E`, wiring each case into `bench`.
pub fn run_interop_cases<E: InteropEngine>(
    mut bench: impl FnMut(&'static str, &mut dyn FnMut()),
) -> Result<()> {
    for case in INTEROP_CASES {
        let Some(mut engine) =
            E::setup(case).with_context(|| format!("setting up interop case `{}`", case.name))?
        else {
            continue;
        };
        crate::rng::reset();
        engine
            .validate(case)
            .with_context(|| format!("validating interop case `{}`", case.name))?;

        bench(case.name, &mut || {
            crate::rng::reset();
            engine.run().unwrap_or_else(|error| {
                panic!("interop case `{}` failed: {error:#}", case.name)
            });
        });
    }

    Ok(())
}

/// Every interop benchmark case.
pub static INTEROP_CASES: &[InteropCase] = &[
    calls::CALL_EMPTY,
    calls::CALL_ONE_I64,
    calls::CALL_THREE_I64,
    calls::CALL_FIVE_I64,
    calls::CALL_F64,
    calls::CALL_STRING_IN,
    calls::CALL_STRING_OUT,
    call_in::CALL_IN,
    callbacks::CALLBACK,
    errors::ERROR_CAUGHT,
    errors::ERROR_PROPAGATES,
    userdata::METHOD,
    userdata::FIELD_GET,
    userdata::FIELD_SET,
    userdata::CTOR_COPY,
    userdata::CTOR_RC,
    bulk::F64_IN_1K,
    bulk::F64_IN_100K,
    bulk::F64_IN_1M,
    bulk::F64_OUT_1K,
    bulk::F64_OUT_100K,
    bulk::F64_OUT_1M,
    bulk::U8_IN_1K,
    bulk::U8_IN_100K,
    bulk::U8_IN_1M,
    bulk::U8_OUT_1K,
    bulk::U8_OUT_100K,
    bulk::U8_OUT_1M,
    bulk::F64_IN_BUFFER_1K,
    bulk::F64_IN_BUFFER_100K,
    bulk::F64_IN_BUFFER_1M,
    bulk::F64_OUT_BUFFER_1K,
    bulk::F64_OUT_BUFFER_100K,
    bulk::F64_OUT_BUFFER_1M,
    bulk::U8_IN_BUFFER_1K,
    bulk::U8_IN_BUFFER_100K,
    bulk::U8_IN_BUFFER_1M,
    bulk::U8_OUT_BUFFER_1K,
    bulk::U8_OUT_BUFFER_100K,
    bulk::U8_OUT_BUFFER_1M,
    serde::TO_SCRIPT,
    serde::FROM_SCRIPT,
];
