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
| NH01 | model 3: Choice + 3 Nouls (question, negative, positive), exact vocab 1,291, qmark token, 100 steps (12 min) | mlpl-repl 0.22.0 | 41,905 | 41,905 | 2,466,150 | 4 | ~5 (browser) | probes 71% after policy; question Noul 12/13 | - | - | - | - | - | escalates 19% (EH01 33%) |
| RC01 | model 4: Choice over 35 DOCTOR rules + 3 Nouls (masked labels), exact vocab 2,000, qmark token, minibatch 256, 606 steps (5 min) | mlpl-repl 0.22.0 | 65,286 | 65,286 | 2,550,960 | 4 | ~6 (browser) | val agreement with the oracle 84.9% (probes 93.8%) | - | - | - | 0.169 | - | +54.8 pts over the majority class on the same split |

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

