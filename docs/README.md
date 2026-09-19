# Documentation

A tiny non-generative **Typed Decision Model** in sw-MLPL, and the microscope
that shows it deciding. The subject is three primitives — **Choice**, **Noul**,
**Scale** — plus **Memory** held outside the model. ELIZA is demo 01, a forcing
function with a free deterministic oracle, not the point.

## Start here

- [`plan.md`](plan.md) — the delivery plan and the authority for scope: the
  primitives, the architecture rule, the measurement contract, ten sagas with
  stated exits, and the non-goals.
- [`research.txt`](research.txt) — the source design discussion the plan was
  distilled from.

## Reference

- [`reference/results.md`](reference/results.md) — one row per lesson run. Every
  quality claim in this repository cites a row here.
- [`reference/sw-mlpl-findings.md`](reference/sw-mlpl-findings.md) — language and
  host gaps met while building, each with a reproducer under `probes/`.

## Implementation

- [`implementation/rust-conventions.md`](implementation/rust-conventions.md) — what the
  microscope crates honour, and the one `sw-checklist` deviation we accept.
- [`implementation/cross-repo-handoffs.md`](implementation/cross-repo-handoffs.md)
  — work belonging to a sibling repository, written down rather than committed
  there.

## The trace format

- [`../schemas/decision-trace-v1.schema.json`](../schemas/decision-trace-v1.schema.json)
  — what a run leaves behind. Renderer- and application-neutral, with two
  invariants that carry the project's claim: exactly three decision kinds, and
  every output string reconstructible from a table the trace itself carries.
- [`../crates/tdm-trace`](../crates/tdm-trace) — the Rust parser and validator.
- [`../fixtures/traces/example-turn-v1.json`](../fixtures/traces/example-turn-v1.json)
  — one worked turn, hand-written before any model existed so the format was
  fixed first. Illustrative numbers, not measurements.

## Reading the contract

`lib/decision.mlpl` is the one file every later saga depends on: the `Decision`
record, the three primitive constructors, and the definitions of `confidence`,
`margin`, and `expectation`. It is pinned by `tests/test_decision.mlpl` so the
shape cannot drift underneath a lesson that has already been measured.
