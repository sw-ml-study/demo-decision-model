# DC01 (PR05): choices supplied as text, and what that does not buy

A fixed-head Choice has one column per label, so it can only choose among the
labels it trained on. PR05 asks whether a candidate can instead be **read**:

```
score_i = f(h_state, h_question, h_card_i)
```

One shared encoder reads the input, the question, and every candidate *card*.
Adding a candidate is then a matter of writing one down, not of retraining — in
principle. This lesson measures whether that principle survives contact with
data.

It was asked for by a downstream project
([`downstream-requests.md`](../reference/downstream-requests.md)) whose reranker
chooses among resource cards that change whenever someone publishes a post. The
honest answer matters more there than a flattering one.

## The shape

`lib/scorer.mlpl`, demo-neutral, five parameter arrays:

| | |
|---|---|
| `sc_E` | the shared text embedding, 2,401 × 32 — one encoder for inputs, questions and cards |
| `sc_Ws`, `sc_Wq` | project the pooled input and the pooled question; their sum, through `tanh`, is the query |
| `sc_Wc` | projects a card; the result is scaled to unit length |
| `sc_g` | one learned scalar: how sharp the cosine may become |

A row's score over the cards is `sc_g * cos(query, card)`, masked so that a row
is scored only over the cards it was offered. That mask is what lets one batch
mix choice sets of different sizes: **79,905 parameters answer two different
questions over two different card sets.**

Demo 01 supplies the tuples. Question 0 is *which rule should answer this*, over
51 cards, one per rule of the DOCTOR script, each written out as its keyword,
its decomposition pattern and its reassemblies. Question 1 is *what is true of
this input*, over four cards: the three Nouls and "none of these". 8,336
training tuples, 933 validation.

**Twelve rules are held out entirely**: their cards are never offered during
training, and all 479 rows they answer are removed from it. At evaluation their
cards are offered like any other. A fixed head cannot even be asked this
question, which is the point.

## Results

Default snapshot, 5 minutes of measured training (674 minibatch steps, 21
epochs). Chance in a five-card field is 20%.

| Rows | n | full field (51 cards) | five cards, four trained | five cards, all held out |
|---|---:|---:|---:|---:|
| cards that trained (validation) | 812 | **85.3%** | 97.4% | 95.9% |
| cards held out of training | 479 | **0.4%** | 36.5% | 18.6% |

### The generality cost, which is the cheap half of the answer

On the 812 validation rows model 4's fixed head has a column for:

| | accuracy |
|---|---|
| model 4, fixed head over 35 merged classes | 86.1% |
| the card scorer, over all 51 unmerged rules | 85.3% |

**About one point**, and the scorer is answering the harder question — 51
unmerged rules against 35 merged classes — with one set of weights that also
answers a second question entirely. Generality is close to free *where the
candidates are ones it trained on*.

### The part that does not work

0.4% on cards held out of training is not a soft result. In a full field the
right-but-unseen card essentially never wins; its mean rank is 17.9 of 51,
against 0.56 for a card that trained. Narrowing the field to five helps
(36.5%), and against four other *equally cold* cards it drops to chance
(18.6%) — so among candidates that all lack training, the model cannot tell
which one fits.

Two things were tried and did not rescue it:

- **Plain inner product** (no normalization): 0.0% in the full field. A card
  that trained as an answer accumulates magnitude no unseen card can match, so
  normalization is necessary — it is just not sufficient. The learned scale
  `sc_g` exists to give the cosine back its sharpness.
- **Tying the card projection to the state projection**, so both live in one
  space: 0.6% held out, and 71% on seen cards instead of 86%. Worse at both.

### Why this data is a hard case, and what it does and does not say

A DOCTOR rule card is a *pattern and its replies* — "i, i am, is it because you
are that you came to me". The input is "I am depressed". The words they share
are function words; what makes the rule correct is a decomposition pattern and
a rank ordering, not topical overlap. Worse, holding out a rule removes its
inputs too, so the evaluation rows are off-distribution twice over, and a
trained catch-all card (`i#11`, "you say ...?") is a genuinely defensible answer
to most of them.

So this is a **lower bound, measured in an unfriendly regime**, and it should
not be read as "card scorers do not generalize". The regime where they are
known to work is the one where a query and a card share content words — a
question about sleep against a card whose title and abstract are about sleep.
That is the downstream project's regime, not ours.

What the result does say, and what a downstream reader should take:

1. **The claim "a candidate written after training can be chosen" needs its own
   measurement on its own corpus.** It is not a property of the architecture.
   The harness shape here — hold candidates out entirely, then report full
   field, small warm field, and small cold field — is reusable, and the small
   cold field is the diagnostic that separates "weak" from "biased".
2. **Where candidates did train, the generality costs about a point.** That
   part transfers.
3. **Nothing was silently absorbed into the label set.** The 12 held-out rules
   include all six that model 4 had to merge away for want of examples; the
   scorer at least offers them.

## What it cost to build

Three `grad` restrictions, all now recorded as finding Q5 with a reproducer:

- a **record field read inside `grad`** is unsupported, and one call deep it
  reports "the loss does not depend on `<param>`" — blaming the parameter while
  training quietly does nothing. Every differentiated entry point in `lib/` now
  takes plain arrays, which is why `u:sc_train` has twelve arguments.
- **`shape()` inside `grad`** is unsupported, so `u:sc_unit(a, k)` takes the row
  count it could otherwise ask for.
- full batch over 8,336 rows is unaffordable in the interpreter, so training is
  one minibatch of 256 per call, continued through Adam's persistent moments.

Parity with the Rust port is exact to `1e-9` over the bundle's parity rows
(`crates/tdm-model/tests/scorer_bundle.rs`), computed on the rounded weights a
consumer receives.
