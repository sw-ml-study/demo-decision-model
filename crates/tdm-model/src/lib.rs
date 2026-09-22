//! Inference for a trained typed decision model, exported from MLPL as a bundle.
//!
//! MLPL is the only trainer in this repository. This crate ports exactly the
//! inference half — the featurizer and the forward pass — so a trained model can
//! run in a browser with no interpreter. The bundle carries a parity set of
//! inputs with the probabilities MLPL computed for them, and the tests require
//! this port to reproduce them.
//!
//! Like `tdm-trace`, this crate knows about decisions and nothing about any
//! particular demo: labels, replies, and keyword lists are data in the bundle.

mod bundle;
mod conversation;
mod features;
mod keyword;
mod matcher;
mod model;
mod responder;

pub use bundle::{Bundle, BundleError, Escalation, Nouls, Snapshots, Weights};
pub use conversation::{
    Conversation, Exchange, Move, NoulRule, NoulRules, Remembered, Reply as ScriptReply, Rule,
    Script,
};
pub use features::{Features, featurize, featurize_vocab, hash, tokens, words};
pub use keyword::{Answer, Decomp, Key, KeywordEngine, KeywordScript, render as render_frame};
pub use matcher::KeywordMatcher;
pub use model::{Decision, Model};
pub use responder::{Outcome, Reply, Turn, respond, respond_at};
