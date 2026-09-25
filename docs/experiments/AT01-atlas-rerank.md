# AT01: the sw-atlas hybrid, measured on sw-atlas's own data

A downstream project, [sw-atlas](../reference/downstream-requests.md), proposes a
*hybrid docent*: a deterministic matcher proposes candidate resources, and a
typed decision model from this repository reads intent, answers a few Nouls, and
reranks the candidates. Its design rests on one claim in particular —

> a new post published last night … the rerank head scores cards, not labels,
> so it can pick it without retraining

— which [`DC01`](DC01-dynamic-choice-sets.md) measured failing badly here, on
cards that were 1966 decomposition rules. That was an unfriendly regime, and the
write-up said so. This is the same question asked in **their** regime, on their
data, before they build on it.

It is also [`NV01`](../plan.md): demo 02, a second domain, with `lib/`
unchanged. What `lib/` needed in order to survive contact with it is the first
finding below, and it is the kind of thing this test exists to find.

## What was measured

Read-only from an sw-atlas checkout; nothing of theirs is committed here.

- **642 resources** — posts, repositories, papers, videos, campus exhibits —
  each with a title, aliases, concepts and a hand-written summary. The card is
  all four, concatenated.
- **386 questions** from six frozen sets. The 308 paraphrases are written so
  that *no title word and no alias of the answer appears in them*: a question
  that contains its own answer measures lookup, not understanding.
- **72 resources held out entirely.** Their cards are never offered during
  training and none of their questions are trained on, so the 91 questions
  pointing at them are exactly the "published last night" case.

Two programs, both using `lib/` unchanged: `demos/atlas/train.mlpl` (the rerank
scorer, 77,985 parameters) and `demos/atlas/intent.mlpl` (the intent Choice and
two Nouls, 17,351 parameters).

**The question sets were marked `Unconfirmed` when this ran.** They had been
drafted by a model and not yet passed over by their author; sw-atlas confirmed
all 386 rows the following day, so the numbers below are against what has since
become a real yardstick. Read the direction rather than the decimals anyway:
the splits are small.

## Finding 1: a guard after `sqrt` is not a guard

The first run produced an accuracy of 0.000 and a mean reciprocal rank of
exactly 1.000 — impossible together, and the signature of comparisons against
`NaN`. `lib/scorer.mlpl` normalized each card to unit length like this:

```
n = sqrt(reduce_add(a * a, 1));
a / reshape(n + eq(n, 0), [k, 1])
```

Forward, that is correct. Backward it is not: the derivative of `sqrt` is
unbounded at zero, so a single all-zero row — **a card whose every word is
unknown to the vocabulary**, which a 642-card catalog has and twelve
hand-written ELIZA rules never did — turns every parameter into `NaN` on the
first Adam step. Training ran to completion and printed results.

The fix is an epsilon inside the root. Recorded as `Q6` in
[`sw-mlpl-findings.md`](../reference/sw-mlpl-findings.md) with a reproducer, and
pinned in the capability probes.

**This is the NV01 dividend.** One library change was forced by the second
domain, it was a real defect, and it would have reached sw-atlas silently.

## Finding 2: the rerank head does not work at this data scale

The matcher is an IDF-weighted token overlap over the same card text — a stand-in
for sw-atlas's `MB02`, not their implementation.

| on 43 warm / 91 cold questions | accuracy@1 | MRR | recall@5 | recall@20 |
|---|---:|---:|---:|---:|
| matcher, warm | 0.233 | 0.329 | 0.442 | 0.651 |
| matcher, cold | 0.209 | 0.325 | 0.462 | 0.626 |
| scorer over the whole catalog, warm | 0.000 | 0.014 | — | — |
| scorer over the whole catalog, cold | 0.000 | 0.018 | — | — |
| scorer reranking the matcher's top 20, warm | 0.023 | 0.112 | — | — |
| scorer reranking the matcher's top 20, cold | 0.032 | 0.114 | — | — |

Reranking twenty candidates at random would score 0.050. **The trained scorer is
worse than chance at its own job**, and far worse than the matcher it was meant
to improve. Training loss reached 0.011: it memorized its 174 training questions
and transferred nothing.

There is no mystery here. One hundred and seventy-four labelled questions cannot
fit a 78,000-parameter embedding space over a 2,340-word vocabulary. The warm
and cold columns are equally bad, which says the problem is not the held-out
cards — it is that the model has not learned a query-to-card space at all.

## Finding 3: intent and the Nouls do not beat their trivial baselines either

sw-atlas's history says matchers are weak at intent (0.48 in its scoreboard), so
this is the half the model is supposed to own. On 76 held-out questions from the
six sets, with 319 for training:

| | model | trivial baseline |
|---|---:|---:|
| intent, five classes | 0.697 | **0.737** (always `FindResource`) |
| is-meta Noul | 0.960 | 0.947 (always false) |
| is-off-topic Noul | 0.947 | **0.947** (always false) |

Per intent: `FindResource` 0.750 over 56, `Navigate` 0.625 over 8, `Explain`
0.750 over 4, `Meta` 0.750 over 4, `Unsupported` **0.000** over 4. The
off-topic Noul learned to say "no" to everything, which on four positives out of
76 is the optimal lazy answer.

Four test examples cannot measure a Noul. What the table supports is narrow and
still useful: **nothing here beat a constant answer.**

## What this means for the hybrid

The design is not refuted. What is refuted is the assumption that it can be
trained from the question sets that exist today.

1. **The hybrid's ceiling is the matcher's recall.** Reranking the top 20 caps
   accuracy at 0.63 on this set no matter how good the reranker becomes. If the
   target is higher, the first stage has to improve, or *k* has to grow — and a
   larger *k* is a harder reranking problem, not an easier one.
2. **The rerank head needs far more supervision than 308 questions.** The
   obvious next experiment, not yet run: synthesize positives from the cards
   themselves — a card's title and its summary as pseudo-queries — to teach the
   shared space, then fine-tune on the real questions. That is the standard
   recipe for a two-tower retriever with sparse labels, and it multiplies the
   training pairs by about ten without a single new hand-written question.
3. **Intent may be the wrong thing to learn from these sets.** 72% of the
   paraphrases are `FindResource`. A set drafted to test *destinations* is not a
   set that teaches *intent*, and the intent labels came along for the ride.
4. **The measurement to run before building:** how many labelled questions per
   resource are needed before the scorer beats the matcher on the same split.
   That number, not an architecture, decides whether the rerank stage is worth
   its place.

## What happened next, on their side

sw-atlas received this and acted on it within a day, which is recorded here
because the value of a measurement is what it changes:

- **The defect was accepted and the vendoring pinned** at or after `b78a2e1`.
- **The rerank head moved behind a supervision step.** Their Saga 3 now begins
  with synthetic positives — a card's title and summary as pseudo-queries —
  before the rerank head is trained at all. That experiment is *theirs*, and is
  deliberately not repeated here.
- **The design changed shape.** After `DC01` they had already rewritten
  `hybrid-docent.md` so that concepts, not a scorer, carry a newly indexed
  resource: every head is a fixed label set, reranking is confined to
  candidates the model trained on, and cold candidates keep the matcher's
  order. The cold-card question this experiment asked is, for them, now closed
  by design rather than by measurement.
- **The ceiling finding became a headline.** Their Saga 2 exit criterion is now
  `MB02`'s own recall@k rather than its accuracy, and they have since measured
  it with their real matcher.
- **The baseline criticism was taken further than it was made.** Their harness
  now prints a majority-class baseline beside every accuracy, and an
  intent-balanced question supplement is queued as owner work: 78 intent rows
  where a constant answer scores 18% rather than 74%.

## Honest limits of this experiment

- The matcher is ours, not sw-atlas's `MB02`; a better first stage changes every
  row of the table.
- The question sets were `Unconfirmed` drafts at the time, and their matcher
  did not exist yet.
- One configuration was tried for each head: 250 steps at lr 0.02 for the
  scorer, 300 at 0.05 for intent. No sweep, no early stopping, no dropout, no
  synthetic positives. A negative result from one configuration is a weaker
  claim than a negative result from a search, and this is the former.
- 43 warm and 91 cold questions; 76 for intent, of which 4 are `Unsupported`.

The one result robust to all of that is Finding 1, which is a defect, and the
ceiling in Finding 4.1, which is arithmetic.
