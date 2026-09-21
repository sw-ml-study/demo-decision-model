# Results

One row per lesson run. Every quality claim in this repository cites a row here,
keyed by lesson ID, configuration, and the sw-MLPL binary version that produced
it.

A row is only written by a lesson's own gate script, never by hand. The three
measurement axes are required by the visual and measurement contract in
[`../plan.md`](../plan.md): no row is complete without memory, speed, and
quality, and no quality claim ships without its per-category breakdown, its
calibration numbers, and its margin over the `MB01` matcher on the same split.

| Lesson | Config | Binary | Params | Active | Bytes | Decisions/pass | us/decision | Accuracy | Top-2 | NLL | Brier | ECE | Coverage@.90 | Margin vs MB01 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| SL01 | 1 Choice, 9 classes, 1024 slots x 32 dim, 200 steps | mlpl-repl 0.22.0 | 33,065 | 33,065 | 264,688 | 1 | ~1,300 | val 0.879 | - | - | - | - | - | +0.483 |
| EH01 | 17 classes + NONE, exact vocab 1,173, dropout 0.3, smoothing 0.1, lr 0.02, 100 steps (5 min) | mlpl-repl 0.22.0 | 38,129 | 38,129 | 2,157,264 | 1 | ~5 (browser) | probes 69% (strict 62%) after policy | - | - | - | - | - | vs v1 model: +29 pts |

## Reading the SL01 row

`SL01` is the thin end-to-end slice, marked `provisional`: it exists so that
something runs, and it will be superseded by `PR01` when the Choice primitive is
built properly. Its dashes are real gaps, not omissions:

- **Top-2, NLL, Brier, ECE, coverage are not measured.** The slice trains to
  near-zero loss and reports `1.000` confidence on most inputs, so its
  probabilities are not usable as probabilities yet. Measuring that honestly is
  `CB01`'s job, and quoting a coverage number from this model would be
  meaningless.
- The repository's rule is that no accuracy claim ships without its per-category
  breakdown, its calibration numbers, and its margin over the matcher. Two of
  three are here; **the calibration numbers are knowingly deferred**, and that is
  recorded rather than quietly skipped.

Per-class and out-of-distribution numbers are in the `SL01` catalog entry and in
the README. The short version: the learned Choice beats the 1966 keyword list on
every split, and is confidently wrong on inputs unlike anything it trained on.

