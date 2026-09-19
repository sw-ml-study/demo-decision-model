//! Parsing and validation for the typed decision trace interchange format.
//!
//! A trace is what a decision model's run leaves behind: for every turn, the
//! state that went in, the choices that were offered, the typed decisions that
//! came out, the policy rule that consumed them, and the output that resulted.
//!
//! This crate knows about *decisions*. It does not know about any particular
//! application: label names, policy text, and output strings are data carried in
//! the trace, never types declared here. That is what lets one instrument render
//! a trace from any demo without a line of new Rust.
//!
//! Two invariants are worth naming, because they are the reason the format
//! exists rather than being incidental checks:
//!
//! 1. **There are exactly three decision kinds.** `choice`, `noul`, and `scale`.
//!    A trace claiming a fourth is rejected. The absence of a generation
//!    primitive is the central claim of the project, so the parser enforces it.
//!
//! 2. **Output text must be reconstructible from a table the trace carries.**
//!    Every turn names a table and an index, and the text must equal that entry
//!    with its `{}` placeholders filled, in order, by the turn's recorded slots.
//!    An entry with no placeholder must match byte for byte. A trace whose
//!    output cannot be rebuilt this way is rejected as
//!    [`ValidationError::TextNotInTable`].
//!
//!    Slots exist because a canned frame is not always a whole sentence: an
//!    application may splice quoted input back into it (`"Earlier you said that
//!    {}."`). That is still not generation — the frame is a fixed literal and
//!    the slot is text the program already had — but pretending the output were
//!    byte-identical to a table entry would be a claim the format could not
//!    honestly make. Rebuilding it exactly is the claim that holds.

mod decision;
mod error;
mod limits;
mod trace;
mod turn;
mod validate;

pub use decision::{Calibration, Decision};
pub use error::ValidationError;
pub use limits::KINDS;
pub use trace::{Provenance, Table, Trace};
pub use turn::{Memory, Output, Policy, State, Turn};
