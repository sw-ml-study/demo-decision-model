# sw-MLPL findings

Language and host gaps met while building this repository. Every entry has a
standalone reproducer under `probes/`, a pinned mlplunit probe, a workaround in
use, and — where the gap is a genuine language blocker — a work order for
[`../../../sw-mlpl`](https://github.com/sw-ml-study/sw-mlpl).

Findings are filed in the same step the gap is met, not batched up later. A gap
without a reproducer is a rumour.

## Open questions awaiting a probe

Scheduled for step `002-language-probes`, before any lesson depends on them:

| # | Question | Why it matters | Fallback if the answer is no |
|---|---|---|---|
| Q1 | Is `freeze` available in `mlpl-repl 0.22.0`? It is absent from the language reference but referenced in peer plans. | The calibration lessons fit a post-hoc map over a frozen encoder. | Recompute the state representation outside `grad` as a stop-gradient idiom. |
| Q2 | Does a gradient flow through a two-argument scorer `f(h_state, h_choice)` built from `matmul` and `concat` inside `grad`? | The dynamic-choice-set primitive (`PR05`) is exactly this shape. | Score choices through a single concatenated input row instead of a two-argument function. |
| Q3 | Do `experiment` blocks compose with `train` inside a user function? | The sweep lessons (calibration lambda, encoder ladder) depend on it. | Drive sweeps from the demo top level and record rows manually. |

## Findings

None recorded yet.
