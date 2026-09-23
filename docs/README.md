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

## Literate reading

- [`literate/concise-hello.org`](literate/concise-hello.org) — the short one:
  a typed decision model in 36 annotated lines, trained on a corpus passed in
  on the command line, written against the widely-shared "Jev in 25 lines of
  Python" and counting honestly what each program stands on.
- [`literate/typed-decisions.org`](literate/typed-decisions.org) — the hello
  world: one state, three typed heads and a calibration measurement, over
  message triage rather than ELIZA, in four stages. Start here. Published
  beside the live demo as `typed-decisions.html`, and self-contained enough to
  run in a browser.
- [`literate/demo-decision-model.org`](literate/demo-decision-model.org) — the
  model read as a literate program: a six-block runnable primer, then every
  library and demo source split at function boundaries. It tangles back to the
  committed files byte for byte, and `just check` proves it. Published beside
  the live demo as `literate.html`.

## Reference

- [`reference/results.md`](reference/results.md) — one row per lesson run. Every
  quality claim in this repository cites a row here.
- [`reference/sw-mlpl-findings.md`](reference/sw-mlpl-findings.md) — language and
  host gaps met while building, each with a reproducer under `probes/`.
- [`reference/claims-discipline.md`](reference/claims-discipline.md) — the ten
  rules every page here is written against: name what you are not counting,
  print the evidence against yourself, and let the gate produce every number.
- [`reference/crate-contract.md`](reference/crate-contract.md) — what a project
  outside this repository may pin `tdm-model` and `tdm-trace` to, and what a tag
  promises.
- [`reference/downstream-requests.md`](reference/downstream-requests.md) — what
  projects that depend on this one have asked for, what was delivered, and
  where the answer is.
- [`reference/superseded-steps.md`](reference/superseded-steps.md) — steps the
  plan queued that later work answered by another route, and what was given up
  by taking it.

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
  every output string exactly one of the candidates the program offered.
- [`../crates/tdm-trace`](../crates/tdm-trace) — the Rust parser and validator.
- [`../fixtures/traces/example-turn-v1.json`](../fixtures/traces/example-turn-v1.json)
  — one worked turn, hand-written before any model existed so the format was
  fixed first. Illustrative numbers, not measurements.

## Reading the contract

`lib/decision.mlpl` is the one file every later saga depends on: the `Decision`
record, the three primitive constructors, and the definitions of `confidence`,
`margin`, and `expectation`. It is pinned by `tests/test_decision.mlpl` so the
shape cannot drift underneath a lesson that has already been measured.
