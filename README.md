# demo-decision-model

Tiny models that decide, not talk.

A **Typed Decision Model** (TDM) takes unstructured state in and returns typed
probabilistic decisions out. It has three primitives — **Choice**, **Noul**, and
**Scale** — plus **Memory** that lives outside it, and it deliberately has no
fourth:

```text
generate(prompt) -> String        <-- not present, and never will be
```

This repository builds one in [sw-MLPL](https://github.com/sw-ml-study/sw-mlpl),
small enough that every parameter, probability, and branch can be printed, and
puts it under a microscope. Ordinary program code owns every threshold, every
side effect, and every string a user ever sees. The model reports belief; the
program decides what to do about it. That separation is the whole thesis, and
the repository exists to make it visible and measurable.

It is an open experiment inspired by the publicly described *behavior* of
TypeSafe's Jev / System One Models. It is not a reimplementation of Jev, whose
architecture, training algorithm, and weights are not public. Where we reproduce
something, we reproduce the stated contract and the calibration objective, and
we say which numbers are theirs and which are ours.

## What is on the slide

**ELIZA is demo 01, not the point.** It is a forcing function: the smallest
realistic application that needs all three primitives *and* memory at once, and
it comes with a free, exact, deterministic oracle — the 1966 rules — to label
training data with.

All of ELIZA's words stay canned. The learned model replaces only the
pattern-matching machinery that decides *which* canned response to use.

```text
 "My mother never understands me."
                |
                v
     +---------------------+
     |  typed decision     |      MOTHER  .71
     |  model              |      FAMILY  .17
     +---------------------+      FEELING .07
                |                 DEFAULT .05
                v
        deterministic code
                |
                v
   "Tell me more about your family."
```

The model can choose wrongly. It cannot invent an utterance that is not in the
table — and a gate test asserts that byte for byte on every commit. That is a
sharper demonstration of the "no hallucination" claim than any statement about
JSON schemas: schema conformance does not mean the decision is correct, it means
the output cannot fall outside the permitted space.

Demo 02 (campus navigation) must run with `lib/` unchanged. That is the test of
whether the primitives are real abstractions or ELIZA-shaped ones.

## Status

**Saga 1 in progress.** The decision contract and the repository gate are in;
no model exists yet.

- [`docs/plan.md`](docs/plan.md) — the delivery plan: primitives, architecture
  rule, ten sagas, the measurement contract, and the non-goals.
- [`docs/research.txt`](docs/research.txt) — the source design discussion.
- [`lib/decision.mlpl`](lib/decision.mlpl) — the `Decision` record and the three
  primitives, with `confidence`, `margin`, and `expectation` defined once and
  pinned by 13 tests so the shape cannot drift under a measured lesson.
- `just check` — the gate: structure, the abstraction boundary, documentation
  links, the catalog, MLPL style, tests, probes.

Nothing has been measured yet. Every number in this README will cite a
[results row](docs/reference/results.md) or be removed.

- [`probes/`](probes/) — three sw-MLPL capability reproducers, all green:
  `freeze` holds a frozen encoder at exactly zero delta, a two-argument bilinear
  scorer differentiates to exactly the outer product (so dynamic choice sets
  will train), and an `experiment`-wrapped `train` loop runs inside a user
  function. Caveats found and written down in
  [`docs/reference/sw-mlpl-findings.md`](docs/reference/sw-mlpl-findings.md).

Next: the `decision-trace-v1` schema and the `tdm-trace` crate, then demo 01's
rule oracle and response table.

## What it will look like

The demo opens as an ordinary ELIZA session — type, press enter, ELIZA answers.
Someone can use it without knowing there is a model inside, which is the honest
presentation of the thesis. Flip **trace** on and the same conversation grows an
instrument: every turn becomes steppable, showing the state that went in, the
choices that were offered, and the typed output with confidence bars, plus the
policy rule that consumed it and the table entry it produced.

```text
+-----------------------------------------------------------------------+
| ELIZA                                      [ trace: ON ]  turn 12/20  |
+----------------------------------+------------------------------------+
| YOU:  I don't know. Everything   |  CHOICE  action        10 offered  |
|       just seems difficult.      |    RECALL     ############  .61    |
|                                  |    REFLECT    ####          .18    |
| ELIZA: Earlier you said that     |    FALLBACK   ###           .12    |
|       work has been stressful.   |    conf .61   margin .43   T=1.37  |
|       Tell me more about that.   |                                    |
|                                  |  NOUL  memory_required?  .83       |
| > _                              |  SCALE emotional_intensity  1.27   |
|                                  |  CHOICE recall  M2 .48 / NONE .05  |
+----------------------------------+------------------------------------+
| POLICY  memory_required > .60 AND max(recall) > .40  ->  RECALL_T03   |
+-----------------------------------------------------------------------+
```

`"Earlier you said..."` does not need a context window or a generative model
remembering anything. One decision says *I need memory*; ordinary code retrieves
a bounded candidate set; a second typed decision picks among `{M1 … Mk, NONE}`;
deterministic code writes the sentence. **Decide → Gather → Decide → Act**, with
the program formulating the model's next question.

## What is here

```text
docs/plan.md                 Delivery plan: primitives, sagas, measurement contract
docs/research.txt            The source design discussion
lib/                         MLPL primitives, encoders, calibration, evaluation
                             (knows about decisions; knows nothing about ELIZA)
demos/eliza/                 Demo 01: the rule oracle, response table, policy
probes/                      Standalone sw-MLPL reproducers re-checked by the gate
tests/                       Native mlplunit tests
fixtures/  assets/previews/  Bounded fixtures, pinned traces, committed diagrams
catalog/                     Machine-readable lesson inventory
crates/                      Rust/Yew/WASM: trace types, view model, web shell
schemas/                     decision-trace-v1 JSON schema
scripts/  justfile           Thin gate and tool-selection scripts
```

Directories arrive as their saga does; `crates/`, `schemas/`, and the demos
are still to come.

## Build and run

Prerequisites:

- the adjacent [`../sw-mlpl`](https://github.com/sw-ml-study/sw-mlpl) checkout
  built in release mode (`target/release/mlpl-repl`), or an absolute `MLPL`
  override. The plan is written against `mlpl-repl 0.22.0`;
- `mlplunit` on `PATH`, an absolute `MLPLUNIT` override, or the adjacent
  `../../softwarewrighter/mlplunit/bin/mlplunit` checkout;
- [`just`](https://github.com/casey/just);
- a Rust toolchain with the `wasm32-unknown-unknown` target and `trunk`, for the
  microscope crates;
- [`agentrail`](https://github.com/softwarewrighter) for the development process;
- [`ollama`](https://ollama.com) **only** for the offline teacher track. Every
  other lesson runs without it.

```sh
just            # list the repository tasks
just check      # the full pre-commit gate: formatting, tests, fixture freshness
just tests      # native mlplunit tests
just probes     # re-run the upstream-finding reproducers
```

The teacher track (Saga 7) generates training labels offline from small local
models — `llama3.2:3b`, `qwen3:4b`, and `gemma3:4b`, three families so that
disagreement means something. Soft labels are read from the first generated
token's `top_logprobs` and renormalized over the legal answer set, so the
distribution comes from the teacher's own head rather than from a confidence
number it was asked to state about itself. The teachers are discarded once the
corpus exists; **no generative model is present at inference.**

## Development process

Work is divided into durable Agentrail steps. In each fresh session run
`agentrail next`, then `agentrail begin`; implement only that step; run focused
tests and `just check`; commit source and `.agentrail/` metadata by name; push
`main`; and only then run `agentrail complete`. [`CLAUDE.md`](CLAUDE.md) holds
the full repository protocol and [`AGENTS.md`](AGENTS.md) mirrors it.

## Copyright and license

Copyright (c) 2026 Michael A Wright. See [COPYRIGHT](COPYRIGHT).

Distributed under the [MIT License](LICENSE).
