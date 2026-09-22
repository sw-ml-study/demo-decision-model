# The crate contract: what a consumer outside this repository can pin

`crates/tdm-model` and `crates/tdm-trace` are meant to be depended on from
other projects. This page is what a consumer is entitled to rely on, and what
it is not.

## The pin

Depend on a **tag**, not on `main`. The bundle format and the crate API both
change on `main` between tags.

```toml
[dependencies]
tdm-model = { git = "https://github.com/sw-ml-study/demo-decision-model", tag = "tdm-v0.1.0" }
tdm-trace = { git = "https://github.com/sw-ml-study/demo-decision-model", tag = "tdm-v0.1.0" }
```

Both crates live inside this repository's Cargo workspace; Cargo resolves that
without help. They build from a clean checkout with no local paths, no MLPL, no
build script and no generated code. Their only dependencies are `serde` and
`serde_json`. Edition 2024, MSRV 1.85. Every tag is verified by building a
consumer crate *outside* this repository that parses a committed bundle,
decides on it, and validates a committed trace.

## What a tag promises

- **The bundle `version` field does not change within a tag.** A reader pinned
  at a tag can load every bundle that tag's exporters produce.
- **The parity tolerance the tests enforce is `1e-9`**: the largest absolute
  disagreement between MLPL's own probabilities, carried in the bundle's
  `parity` block, and this crate's forward pass, over every parity input at
  every snapshot. The tests fail if it is exceeded.
- Weights are rounded for transport before parity is computed, so parity is
  measured on exactly the numbers a consumer receives.

## `tdm-model`

Reads a **decision bundle** (`sw-ml-study.decision-bundle`), versions **2 and
3**, and runs the forward pass the trainer measured.

| Version | What it adds |
|---|---|
| 2 | hashed features, one Choice head, a keyword matcher, snapshots |
| 3 | an exact vocabulary, a trained "none of these" label, an escalation policy, a question token, and Noul heads on the shared pooled state |

Entry points: `Bundle::parse`, `Model::at` / `Model::new` (`pooled`, `logits`,
`nouls`, `decide`), `respond` / `respond_at` for the policy, `KeywordScript` /
`KeywordEngine` for a keyword-and-reassembly script, and `Conversation` /
`Conversation::with_keywords` for a stateful session with memory, recall and
deflection. `featurize`, `featurize_vocab`, `tokens`, `words` and `hash` are
public because a consumer that wants to build its own training data must
featurize exactly as the model does.

### Compatibility note, as of `tdm-v0.1.0`

A version 3 bundle may now carry an **empty** `keywords` and `match_order`.
That is a bundle whose labels are another program's rules (demo 01's model 4
chooses among the 1966 DOCTOR script's rules): it has no keyword yardstick of
its own, because the script itself is the yardstick, and its `parity.matcher`
is empty for the same reason. Readers built before commit `b13b1d0` reject such
a bundle with `Shape("per-label data")`. A consumer that must read model-4-shaped
bundles has to pin at or after `tdm-v0.1.0`.

## `tdm-trace`

Parses and validates the decision-trace interchange format
(`schemas/decision-trace-v1.schema.json`). Two invariants are enforced by the
parser rather than left to a convention:

1. **There are exactly three decision kinds** — `choice`, `noul`, `scale`. A
   trace claiming a fourth is rejected. The absence of a generation primitive
   is the central claim of this project, so the format refuses to carry one.
2. **Output text must be reconstructible** from a table the trace carries, with
   the turn's recorded slots filled in order. A trace whose output cannot be
   rebuilt is rejected as `ValidationError::TextNotInTable`.

Entry point: `Trace::parse`.

## What is *not* promised

- No semantic versioning of the Rust API between tags; read the tag notes.
- Nothing under `crates/tdm-web`, `demos/`, `lib/` or `fixtures/` is part of the
  contract. Bundles in `fixtures/bundles/` are demo 01's, retrained whenever a
  lesson lands, and their numbers change.
- No crates.io release. A git tag is the unit of pinning.
