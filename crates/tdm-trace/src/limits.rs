//! The format's fixed identity, bounds, and tolerances.
//!
//! Every budget here exists so that a trace is a bounded artifact: a renderer
//! can size its work from the limits alone, without trusting the producer.

pub(crate) const SCHEMA: &str = "sw-ml-study.decision-trace";
pub(crate) const VERSION: u32 = 1;

pub(crate) const MAX_TURNS: usize = 4_096;
pub(crate) const MAX_DECISIONS_PER_TURN: usize = 64;

/// The highest choice cardinality a single decision may offer. Larger candidate
/// sets go through the two-stage path (score independently, take the top k, then
/// choose) rather than being presented to one softmax.
pub(crate) const MAX_LABELS: usize = 256;

pub(crate) const MAX_TABLES: usize = 64;
pub(crate) const MAX_TABLE_ENTRIES: usize = 4_096;
pub(crate) const MAX_MEMORIES: usize = 256;
pub(crate) const MAX_FEATURES: usize = 256;
pub(crate) const MAX_TEXT_BYTES: usize = 4_096;

/// Absolute tolerance for a probability sum and for every derived scalar.
pub(crate) const TOLERANCE: f64 = 1e-6;

/// The placeholder a table entry uses to mark a spliced slot.
pub(crate) const PLACEHOLDER: &str = "{}";

/// The three decision kinds. There is deliberately no fourth: the absence of a
/// generation primitive is the central claim of the project, so the parser
/// enforces it rather than the documentation merely asserting it.
pub const KINDS: [&str; 3] = ["choice", "noul", "scale"];
