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
