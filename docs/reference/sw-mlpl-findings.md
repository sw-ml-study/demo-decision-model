# sw-MLPL findings

Language and host gaps met while building this repository. Every entry has a
standalone reproducer under `probes/`, a pinned mlplunit assertion in
[`../../tests/test_capability_probes.mlpl`](../../tests/test_capability_probes.mlpl),
a workaround in use, and — where the gap is a genuine language blocker — a work
order for [`sw-mlpl`](https://github.com/sw-ml-study/sw-mlpl).

Findings are filed in the same step the gap is met, not batched up later. A gap
without a reproducer is a rumour.

Verified against `mlpl-repl 0.22.0` on 2026-09-18.

## Answered: the three Saga 1 questions

All three came back green. Nothing is blocked, and no work order is open.
The `sw-mlpl` maintainers answered these independently; the answers below are
what this repository's own reproducers measured, which is what the lessons are
allowed to rely on.

### Q1 — is `freeze` available, and does it hold parameters fixed?

**Yes.** `probes/q1_freeze_holds_params.mlpl` freezes an encoder, then runs five
Adam steps over `[encoder, head]`. The encoder's output delta is exactly `0`;
the head's is `0.495`. The calibration lessons can fit a post-hoc map over a
frozen state representation, as planned. The test asserts the head moved too, so
the check cannot pass vacuously.

`freeze` is absent from the 0.22.0 language reference, which is what raised the
question. It works; the reference is what is out of date.

> **Caveat found here, not in the answer we were given.** `freeze` is a
> *statement*, not an expression: it returns nothing bindable. `fe =
> freeze(enc)` does not fail at the freeze — it leaves `fe` **undefined**, and
> the failure surfaces later as `undefined variable: fe`. Freeze in place and
> keep using the original identifier.

### Q2 — does a gradient flow through a two-argument scorer inside `grad`?

**Yes, exactly.** This was the one flagged as most costly to discover late,
because `PR05`'s dynamic-choice-set primitive is precisely this shape.

`probes/q2_two_arg_scorer_grad.mlpl` differentiates a bilinear scorer
`f(h_state, h_choice) = h_s W h_c^T` with respect to the shared weight `W`. The
gradient is the outer product of the two encoded rows, with total absolute error
`0` — not merely close. Scoring a choice set that did not exist during training
is trainable.

The probe deliberately uses the real shape (rank-2 rows, `matmul`, a shared
`[3, 3]` weight) rather than the scalar form, because the scalar form would have
proved less than the lesson needs.

### Q3 — do `experiment` blocks compose with `train` inside a user function?

**Yes.** `probes/q3_experiment_sweep_in_function.mlpl` wraps `experiment "…" {
train N { adam(…) } … }` in a user function and calls it three times. Each call
runs, returns the block's value, and appends to the experiment log.
`experiment_metric` returns the recorded values as a `[runs]` vector in run
order. The calibration-lambda and encoder-ladder sweeps can each be one callable
per configuration.

> **Two constraints found here that the sweep lessons must design around.**
>
> 1. **The experiment name is a string literal, not an expression.**
>    `experiment name { … }` with `name` a parameter is a parse error
>    (`UnexpectedToken`). Every run of one sweep function therefore shares one
>    experiment name; runs cannot be named dynamically.
> 2. **So runs are told apart by what they record, not by what they are called.**
>    The working idiom, pinned by the probe: record the swept setting as its own
>    `*_metric` beside the outcome. `experiment_metric("lr_metric")` and
>    `experiment_metric("loss_metric")` then come back as aligned columns in run
>    order — `[0.01, 0.2, 0.5]` against `[3.267, 2.083, 0.715]` — which is
>    exactly the sweep table the lessons need.

Also confirmed while checking: `adam` takes the six-argument form
`adam(loss, params, lr, b1, b2, eps)`, and `params` may be a list of models.

## Gate note

`mlpl-repl` exits non-zero on an evaluation error, so `scripts/run-probes`
(which checks exit status) is a real check rather than a vacuous one. Verified
by running a deliberately broken program: `exit=1`.

## Open blockers

None.
