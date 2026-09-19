# Typed decision model delivery plan

## Outcome

Build a tiny, non-generative **Typed Decision Model (TDM)** — small enough that
every parameter, probability, and branch can be printed — together with the
microscope that shows what it is doing.

The subject of this repository is **three decision primitives and one
capability**:

| Primitive | Call | Returns |
|---|---|---|
| **Choice** | `choice(state, question, choices[])` | a probability per choice, the selected id, confidence, margin |
| **Noul** | `noul(state, proposition)` | one probability in `[0, 1]` |
| **Scale** | `scale(state, levels[])` | a distribution over an *ordered* scale, plus its expectation |
| **Memory** | (not a primitive) | structured state the program keeps *outside* the model, which decisions address |

There is deliberately no fourth primitive, and in particular no:

```text
generate(prompt) -> String
```

A TDM takes unstructured state in and emits typed probabilistic decisions out.
Ordinary program code owns every threshold, every side effect, every retrieval,
and every string a user ever sees. The model reports belief; the program decides
what to do about it. That separation is the thesis, and the repository exists to
make it visible and measurable.

**ELIZA is demo 01, not the point.** It is a forcing function: it is the
smallest realistic application that demands all three primitives *and* memory at
once, and it comes with a free, exact, deterministic oracle (the 1966 rules) to
label training data with. It also has a property no other demo has — every word
it speaks is chosen from a bounded set the program built, so "the model cannot
invent an output outside the permitted space" stops being a slogan and becomes
something a test can enforce. When demo 02 arrives (campus navigation, in the
manner of `../moe-microscope`'s docent), nothing in `lib/` should have to change.
That is the acceptance test for whether the primitives are real abstractions or
ELIZA-shaped ones.

The headline question, which every experiment serves:

> Can a tiny non-generative model answer typed Choice, Noul, and Scale questions
> — over dynamically supplied choice sets, with useful **calibrated** confidence
> — well enough that ordinary software can safely branch on the result? And how
> small can it be?

## Naming and attribution

"System 1" is general cognitive-science vocabulary (Kahneman). **"System One
Model"** is TypeSafe's brand for their new model category, introduced with
**Jev**. TypeSafe has published Jev's *behavioral* contract — unstructured state
in, predefined typed outputs with probabilities, many questions answered in
parallel against one state, no autoregressive string generation, a two-stage
path for high-cardinality choices, and RLCD training aimed at calibration — but
not the architecture, the training algorithm, or the weights.

Therefore:

- What we build is a **Typed Decision Model (TDM)**. We never call it a System
  One Model and never claim it reproduces Jev.
- Documentation says: *a small non-generative typed decision model inspired by
  the publicly described behavior of TypeSafe's Jev / System One Models.*
- Our calibration training stage is **calibration fine-tuning**, never RLCD. We
  reproduce the stated *objective* (honest probabilities), not their undisclosed
  algorithm.
- We name the third primitive **Scale** (TypeSafe's public material calls the
  equivalent *Score*); ours returns a distribution over ordered levels plus its
  expectation, and the docs say so rather than implying parity.
- Every claim about Jev in our documentation is quoted from the public post and
  marked as *their claim*, kept separate from *our measurement*.

The source discussion is retained verbatim at [`research.txt`](research.txt).

## The architecture rule

```text
        unstructured state (text, conversation, whatever a demo has)
                              |
                              v
              +-------------------------------+
              |  TDM: shared state encoder    |   the ONLY learned component
              |  + Choice / Noul / Scale      |   one forward pass, no decoder
              +-------------------------------+
                              |
                  typed probabilistic decisions
                              |
                              v
              +-------------------------------+
              |  ordinary MLPL program        |   thresholds, policy, retrieval,
              |  (policy, memory, templates)  |   transforms, table lookup
              +-------------------------------+
                              |
                              v
                 an action, or a string the program owned all along
```

Nothing in this repository may add a generation path. **Every string a user sees
is exactly one of the candidates the program offered the model.** The model
ranks or scores those candidates; it composes nothing.

Two ways to build a candidate set are both legitimate, and demo 01 uses both:

1. **Composed candidates.** Ordinary code assembles the options in full — `Tell
   me more.`, `Why do you say that?`, `Earlier you said you bought a new car.` —
   and one Choice ranks them. The output is byte-identical to the candidate that
   won. Nothing is templated at any point.
2. **Frame plus retrieved text.** Code picks a frame from a response table and
   splices in text it already had (a stored memory, the user's own words after a
   deterministic pronoun transform). The frame is a fixed literal; the slot is
   quoted, never produced.

A gate test asserts the invariant in whichever form a turn used: the text equals
the selected candidate, or it equals the frame with its recorded slots filled.
Either way the central claim is enforced by the build rather than by intention.

## Builtin first, from scratch where it teaches

Prefer an existing sw-MLPL feature over an equivalent built here. Build a
mechanism from scratch only when watching it work is the lesson, and then show
both forms with a parity check. The catalog records each lesson as `builtin`,
`from-scratch`, or `both`.

| Mechanism | Builtin exists | Lesson form | Why |
|---|---|---|---|
| Embedding, linear, chain, ReLU, RMS norm, attention, softmax, sigmoid, cross-entropy, Adam, `train` | yes | builtin | no educational reason to rewrite them |
| Hashed word/bigram features, pooled text encoder | no builtin; technique proven in `../moe-microscope/lib/docent.mlpl` | from-scratch, ported | the feature table is the model's whole input; it must be visible |
| Choice / Noul / Scale primitives and the decision record | no | from-scratch | **this is the subject of the repository** |
| Binary cross-entropy for Noul heads | no builtin; `log`, `sigmoid`, `*`, `mean` are differentiable | from-scratch, one line | every Noul needs it |
| Ordered-scale loss and expectation | no | from-scratch | an ordered scale is not a 3-way classifier; confusing `LOW`/`HIGH` must cost more than `LOW`/`MEDIUM` |
| Classic ELIZA keyword/rank/decomposition/reassembly engine | no | from-scratch | it is the oracle, the baseline, and the bounded output space |
| Temperature scaling | no | from-scratch | one scalar fitted on validation; the cheapest calibration baseline |
| Platt scaling, isotonic regression (PAVA) | no | from-scratch | watching these run *is* the calibration lesson |
| Brier, NLL, ECE, reliability bins, coverage curves | no | from-scratch | the measurements are the product |
| Top-k candidate selection | yes (`grade_down` stable argsort + `gather_rows`) | builtin | one line; do not rewrite argsort |
| Conversation memory store and retrieval | no | from-scratch | "state outside the model" is the lesson |
| Teacher inference | yes (Ollama, local) | builtin | offline data generation; not our research contribution |
| Recording emission and playback plumbing | yes (`emit_frame`, `svg`, the Yew patterns in `../demo-extensions`) | builtin | dogfood, do not rebuild |
| The demo UI and the trace instrument | no | from-scratch (Rust/Yew/WASM) | a deliverable of equal standing to the model |

## Three questions answered up front

### 1. What does a typed question look like, concretely?

The contract is fixed once, in `lib/decision.mlpl`, and every demo and every
lesson uses it unchanged. A decision record:

```text
Decision {
  kind:        "choice" | "noul" | "scale",
  question:    "which earlier statement is worth recalling?",
  labels:      ["M1", "M2", "M3", "M4", "NONE"],   # choice / scale
  probs:       [0.13, 0.48, 0.27, 0.07, 0.05],
  selected:    1,
  confidence:  0.48,          # p of the selected label
  margin:      0.21,          # p1 - p2; low margin is a different doubt than low p
  expectation: 1.27,          # scale only
  calibration: { method: "temperature", t: 1.37 } | none,
}
```

`confidence` and `margin` are both reported everywhere, because they fail
differently: `.48 / .21` is an uncertain decision, `.48 / .01` is a coin flip
between two candidates, and a policy may reasonably treat those differently.

**Demo 01 exercises all three primitives plus memory**, which is why it was
chosen:

- **Choice — `action`** (10 classes): `GREET`, `YES`, `NO`, `DONT_KNOW`,
  `REFLECT`, `ASK_MORE`, `RECALL`, `TOPIC_SHIFT`, `KEYWORD`, `FALLBACK`.
- **Choice — `topic`** (16 classes): `MOTHER`, `FATHER`, `FAMILY`, `SELF`,
  `FEELING`, `DESIRE`, `BELIEF`, `MEMORY`, `DREAM`, `FEAR`, `WORK`,
  `POSSESSION`, `HEALTH`, `COMPUTER`, `SORRY`, `OTHER`.
- **Choice — `recall`** (*dynamic* cardinality): `{M1 … Mk, NONE}`, where the
  candidates are built at call time by ordinary retrieval code and did not exist
  at training time. This is the case that forces choices to be model *inputs*
  rather than fixed output classes.
- **Noul** (independent sigmoids): `mentions_family?`, `mentions_self?`,
  `contains_question?`, `negative_sentiment?`, `short_answer?`,
  `memory_required?`, `repetitive?`, `mentions_computer?`.
- **Scale**: `emotional_intensity` over `LOW | MEDIUM | HIGH`, as a distribution
  and an expectation.
- **Memory**: the structured conversation store, and the `RECALL` loop below.

Decisions are **factored, never a cross-product**.
`RECALL_FAMILY_NEUTRAL_QUESTION` as a single class is the failure mode: it
explodes combinatorially and starves every class of examples. `OTHER`,
`FALLBACK`, and `NONE` are first-class trained classes with deliberate positives
and hard negatives, never leftovers.

### 2. Where does the training signal come from, and what defines truth?

A strict **label authority hierarchy**, highest first:

```text
1. the classic ELIZA rule engine   exact, deterministic, free
2. human-reviewed cases            small; for disputes and the hard set
3. teacher consensus               >= 2 of 3 local models agree
4. a single local teacher          linguistic diversity only
```

This matters more than it looks. The teachers exist to supply paraphrase,
indirection, and semantic generalization — **not** to define what the
application is. Without this hierarchy, "distillation" quietly teaches the
student to be a small modern chatbot instead of ELIZA, and the demo's whole
point evaporates.

Three sources feed one corpus:

```text
        rule oracle          templates            local teachers
        (exact labels)       + mutation           (paraphrase, indirection)
              |                   |                       |
              +---------+---------+-----------+-----------+
                                  |
                        label-authority merge
                                  |
                          canonical soft labels
                                  |
                              train student
```

The corpus is generated by **curriculum category**, so we always know what the
model is being asked to learn:

| Category | What it contains |
|---|---|
| `easy` | explicit keyword; a classic rule hit |
| `paraphrase` | semantic equivalent with the keyword removed |
| `ambiguous` | two genuinely plausible actions |
| `negative` | keyword present, wrong interpretation |
| `memory` | an earlier statement is worth recalling |
| `memory-negative` | nothing worth recalling; `NONE` is correct |
| `topic-shift` | the conversation has gone repetitive |
| `short-input` | `yes` / `no` / `maybe` / `dunno` |
| `adversarial` | misleading keyword overlap |

Every accuracy number in the results table is reported **per category** as well
as overall. An overall 94% that is 99% on `easy` and 41% on `adversarial` is a
different result, and the table must say so.

### 3. What is the value demonstration?

A **pocket decision engine**: a packed file of a few hundred kilobytes with no
generative model anywhere in the deployed path, answering typed questions fast
enough to sit inside a program's control flow.

For demo 01 the evidence is a side-by-side that isolates exactly what the
learned component bought:

```text
                         INPUT
                           |
                  +--------+--------+
                  v                 v
            ELIZA 1966          the TDM
            keyword rules       learned decisions
                  |                 |
            rule selection      probabilities + policy
                  |                 |
                  +--------+--------+
                           v
                  THE SAME RESPONSE TABLE
```

| Input | Classic selector | Learned selector |
|---|---|---|
| `my mother hates me` | `MOTHER` | `MOTHER` (high) |
| `mom never listens` | fallback | `MOTHER` (high) |
| `the woman who raised me worries` | fallback | `FAMILY` (moderate) |
| `I dreamed about mom` | `DREAM` | `DREAM` / `MOTHER` split |

The cleanest single piece of evidence is the **held-out lexicon experiment**:
hold `mom`, `mommy`, `mama` entirely out of training; train on `mother`,
`parent`, `family`; then test `Mom won't leave me alone.` The classic rules fall
through. If the TDM answers `MOTHER` with well-calibrated confidence, the claim
is demonstrated with a number rather than an anecdote.

## Evidence and current constraints

Measured against the adjacent `../sw-mlpl` checkout (`mlpl-repl 0.22.0`) and its
language reference, and against the local Ollama install (`0.32.9`). Nothing
here is an upstream request yet.

**Confirmed available and sufficient:**

- Model DSL: `embed`, `linear`, `chain`, `relu_layer`, `rms_norm`,
  `causal_attention`, `softmax_layer`, `apply`, `predict_batch`, `param_count`,
  `param[shape]` leaves, reverse-mode `grad`, `adam`, `train N { }`,
  `experiment "name" { }`, `experiment_metric`.
- Decision math: `softmax(a, axis)`, `sigmoid`, `cross_entropy(logits, targets)`
  (fused, stable, differentiable), `one_hot`, `argmax(a, axis)`, `log`, `exp`,
  `mean`, `reduce(:add, a, axes)`.
- `gather_rows` is **differentiable** — its backward is a scatter-add into the
  addressed rows — so a from-scratch hashed feature table trains. The whole
  student rests on this.
- `grade_up` / `grade_down` are stable argsorts: exactly the top-k stage of the
  two-stage selector, with `gather_rows(C, grade_down(scores))` as the idiom.
- Text: `tokenize_bytes`, `train_bpe`, `str_len`, `str_slice`, `str_find`,
  `str_join`, `dedupe_rows`, `windows`.
- Observation: `emit_frame`, `svg` (`bar`, `line`, `scatter`, `heatmap`,
  `baseline`), `clock_ms`, `file_size`, `file_metadata`.
- Projection for state-space diagrams: `pca_components`, `mds`.
- **Ollama `0.32.9` returns real per-token log probabilities.** Verified on this
  machine: `POST /api/generate` with `"logprobs": true, "top_logprobs": 5`
  returns, for each generated token, the token and a `top_logprobs` array of
  `{token, logprob, bytes}`. This is what makes honest soft-label distillation
  possible — see the teacher saga.

**Composed here, no builtin (each gets a lesson, not a complaint):**

- Binary cross-entropy for Noul heads (from `log`/`sigmoid`).
- An ordered-scale loss, so adjacent-level errors cost less than distant ones,
  plus the expectation.
- Brier, NLL, ECE, reliability bins, coverage-versus-accuracy curves.
- Temperature scaling, Platt scaling, isotonic regression (PAVA).
- The reliability diagram: composed from `scatter` + `line` + `baseline`; there
  is no `reliability` chart kind.

**To be probed in Saga 1, before anything depends on it:**

- `freeze` does not appear in the 0.22.0 language reference although peer plans
  reference it. Probe `probes/p1_freeze_available.mlpl`; if absent, the
  frozen-encoder calibration lesson uses a stop-gradient idiom (recompute `h`
  outside `grad`) and the finding is filed.
- Gradient flow through a two-argument scorer `f(h_state, h_choice)` built from
  `matmul` + `concat` inside `grad` — the dynamic-choice-set lesson depends on it.
- Whether `experiment` blocks compose with `train` inside a user function, which
  the sweep lessons depend on.

Every gap met gets a reproducer under `probes/`, a pinned mlplunit probe, a
workaround, and an entry in `docs/reference/sw-mlpl-findings.md`, in the same
step it is met.

## Repository boundaries

```text
demo-decision-model (.mlpl primitives, encoders, calibration, evaluation; demos)
          |
          +-- demo UI + trace instrument (Rust/Yew/WASM) --> crates/ in THIS repo
          |
          +-- repeated domain-neutral helpers ------------> ../demo-mlpl-libraries
          |
          +-- generic recording/playback need -----------> ../demo-extensions
          |
          `-- proven language-wide blocker --------------> ../sw-mlpl
```

Sibling repositories are read-only from here; work for them is written as a
handoff in `docs/implementation/cross-repo-handoffs.md`.

Inside this repo the same discipline applies one level down, and it is the test
that the abstraction is real:

```text
lib/            knows about Choice, Noul, Scale, Memory, calibration, encoders.
                Knows NOTHING about ELIZA.
demos/eliza/    knows about ELIZA: keywords, the response table, the policy.
                Owns every ELIZA-specific string and rule.
crates/         know about DECISIONS. No ElizaVisualizer, no hard-coded MOTHER,
                no response table in Rust. Labels, strings, and policy text
                arrive as data in the trace.
```

`sw-checklist` applies to the Rust crates.

## The TDM at two scales

The same MLPL source runs at two documented scales. The microscope scale is the
acceptance scale for every lesson.

| Component | Microscope scale (CPU, seconds) | Lab scale (opt-in, minutes) |
|---|---|---|
| Feature table | 512 hashed word + bigram slots | 4,096 slots, or BPE |
| Features per input | 24 padded (12 words + 11 bigrams) | 64 |
| `d_model` | 32 | 64 to 128 |
| Encoder | mean pool | pool / CNN / GRU / 1-head attention / SSM |
| Choice heads | 32 x 10, 32 x 16 | 128 x N |
| Noul heads | 8 x (32 x 1) | 8 x (128 x 1) |
| Scale head | 32 x 3 | 128 x 3 |
| Dynamic scorer | 32 x 32 bilinear + tiny MLP | 128 x 128 |
| Trainable parameters | roughly 17K to 60K | roughly 270K to 2M |
| Deployment | INT8 packed file | same format |

A 17K-parameter model that handles this decision surface is a *more* interesting
result than a 2M-parameter one, so the ladder starts at the floor and climbs only
when a measurement demands it. The research's illustrative ~274K figure (4K
vocabulary x 64 embedding) is the *upper* end of our starting range, not a target.

## Decide → Gather → Decide → Act

The loop that lifts this above "a classifier with nice types", and the reason
memory is in scope from the start.

```text
                    STATE
                      |
                      v
           +----------------------+
           |  Decision #1         |  choice(state, "what should happen?", [...])
           +----------+-----------+
                      |
               RECALL = .84
                      |
                      v
           +----------------------+
           |  ORDINARY CODE       |  cheap retrieval over the memory store:
           |  gathers candidates  |  keyword overlap, topic tag, recency
           +----------+-----------+
                      |
            M1  M2  M3  M4  NONE     <-- built at call time; never seen in training
                      |
                      v
           +----------------------+
           |  Decision #2         |  choice(state, "which memory helps?", [...])
           +----------+-----------+
                      |
                  M3 = .62
                      |
                      v
           +----------------------+
           |  ORDINARY CODE       |  transform + template + table lookup
           +----------+-----------+
                      |
                      v
        "Earlier you said that work has been stressful."
```

The *program*, not the model, decides more information is needed, gathers it,
and formulates the next typed question. The model never sees the whole
conversation, so conversation length does not grow model cost — the contrast
with stuffing a context window is measured, not asserted.

Memory is a plain record, extracted deterministically:

```text
Memory { turn: 1, text: "My mother worries about me.",
         keywords: ["mother", "worries"], topic: FAMILY, age_turns: 11 }
```

`NONE` is always in the candidate set. Forcing a model to pick the least-wrong
memory out of `{cereal, rain, printer, dog}` is the bug; a well-trained
`NONE .88` is the fix; and `if NONE > .60 { don't recall }` is application
policy, sitting in MLPL where it can be read and changed.

## The demo UI and the trace instrument (Rust / Yew / WASM)

**The UI is a conversation first.** After the model is trained, the page opens as
an ordinary ELIZA session: a plain terminal-styled chat, type and press enter,
ELIZA answers. Someone can use it without knowing there is a model inside. That
is the honest presentation of the thesis — the typed decision engine is
*invisible in normal operation*, which is exactly the claim.

**Tracing is a toggle.** Flip it on and the same conversation grows an
instrument: every turn becomes steppable, and each decision that produced it is
laid out as state in, choices in, and typed output with confidence bar charts,
in the manner of the research's diagnostic-instrument sketch.

```text
+-----------------------------------------------------------------------+
| ELIZA                                      [ trace: ON ]  turn 12/20  |
+----------------------------------+------------------------------------+
| YOU:  My mother doesn't          |  << TRACE  [<] [>] [||]            |
|       understand me.             |                                    |
|                                  |  STATE IN                          |
| ELIZA: Tell me more about        |    utterance  "my mother doesn't.."|
|       your family.               |    features   24 of 512 slots      |
|                                  |    memory     11 turns stored      |
| YOU:  I don't know. Everything   |                                    |
|       just seems difficult.      |  CHOICE  action        10 offered  |
|                                  |    RECALL     ############  .61    |
| ELIZA: Earlier you said that     |    REFLECT    ####          .18    |
|       work has been stressful.   |    FALLBACK   ###           .12    |
|       Tell me more about that.   |    ASK_MORE   ##            .09    |
|                                  |    conf .61   margin .43   T=1.37  |
| > _                              |                                    |
|                                  |  NOUL                              |
|                                  |    memory_required?   .83  ####### |
|                                  |    short_answer?      .71  ######  |
|                                  |    contains_question? .02  .       |
|                                  |                                    |
|                                  |  SCALE  emotional_intensity        |
|                                  |    LOW .08  MED .57  HIGH .35      |
|                                  |    expectation 1.27                |
|                                  |                                    |
|                                  |  CHOICES IN (built by code)        |
|                                  |    M1 "my mother worries"          |
|                                  |    M2 "work has been stressful"    |
|                                  |    M3 "I can't sleep"   M4 ...     |
|                                  |    NONE                            |
|                                  |                                    |
|                                  |  CHOICE  recall         5 offered  |
|                                  |    M2         ###########   .48    |
|                                  |    M3         ######        .27    |
|                                  |    NONE       #             .05    |
+----------------------------------+------------------------------------+
| POLICY  memory_required > .60 AND max(recall) > .40  ->  RECALL_T03   |
| OUTPUT  response table "recall" entry 3, verbatim                     |
+-----------------------------------------------------------------------+
```

Always three things per decision: **state in**, **choices in**, **typed output**
with a bar chart. Plus the policy rule that consumed it and the table entry it
produced, so the whole chain text -> probabilities -> code -> canned text is on
one screen.

Crates in this repository:

```text
crates/tdm-trace/        Decision-trace types, serde, schema validation. No UI.
crates/tdm-instrument/   Headless view model: turns, stepping, selection,
                         threshold overlays, reducer + transitions. No Yew.
crates/tdm-web/          Yew/WASM shell: chat view, trace view, nothing else.
schemas/decision-trace-v1.schema.json
```

This follows the `mlpl-microscope-model` / `mlpl-microscope-web` split already
proven in `../demo-extensions`: the reducer and view model are headless and unit
tested in plain Rust; the Yew crate is a thin renderer. That is what keeps the
instrument testable without a browser.

MLPL lessons emit a `decision-trace-v1` document per run, pinned by hash under
`fixtures/traces/`, carrying a `provenance` block (producer, producer revision,
generation time, source description) in the manner of
`../demo-extensions/schemas/system-layout-v1.schema.json`. One turn:

```json
{
  "turn": 12,
  "state": { "utterance": "...", "features": [...], "memory": [...] },
  "decisions": [
    { "kind": "choice", "question": "action", "labels": ["GREET", "..."],
      "probs": [...], "selected": 6, "confidence": 0.61, "margin": 0.43,
      "calibration": { "method": "temperature", "t": 1.37 } },
    { "kind": "noul", "proposition": "memory_required?", "p": 0.83 },
    { "kind": "scale", "scale": "emotional_intensity",
      "levels": ["LOW", "MEDIUM", "HIGH"], "probs": [...], "expectation": 1.27 },
    { "kind": "choice", "question": "recall",
      "labels": ["M1", "M2", "M3", "M4", "NONE"], "probs": [...],
      "selected": 1, "confidence": 0.48, "margin": 0.21 }
  ],
  "policy": { "rule": "memory_required > .60 AND max(recall) > .40",
              "branch": "RECALL_T03" },
  "output": { "table": "recall", "index": 3, "text": "Earlier you said..." }
}
```

Two run modes, one UI:

- **Playback (default, always available).** No MLPL runs in the browser; every
  number comes from a pinned trace a real run wrote. This is what ships on Pages.
- **Live (opt-in).** The page submits turns to `mlpl-serve` over the SSE path
  proven in `../demo-extensions` and streams fresh traces. Same view model, same
  renderer.

An acceptance test asserts that `output.text` is exactly one of the candidates
the program offered: for `source: "choice"`, byte-identical to the selected label
of the decision it cites; for `source: "table"`, equal to the frame at
`output.index` with its recorded slots filled. The instrument cannot display a
string the program did not construct, and neither can the demo.

## Visual and measurement contract

Applies to every lesson in every saga.

**Every data structure and transformation gets a diagram.** The feature table,
the pooled state, each head's logits, each distribution, the memory store, the
candidate set, the calibration map, the policy branch — drawn once with shape,
dtype, byte size, and axis meanings; each transformation drawn as
before -> operation -> after with the fixture's exact values on the arrows.
Diagrams are generated from the same recorded values the tests assert on. Static
SVG under `assets/previews/` is the canonical committed form. Color is never the
only carrier of meaning; every diagram has a title, a description, and a numeric
table beside it.

**Every lesson measures three axes**, recorded in its catalog entry:

| Axis | Required measurements | How |
|---|---|---|
| Memory | trainable parameters, active parameters per decision, feature-table bytes, packed-file bytes, memory-store bytes, trace bytes | `param_count`, shape arithmetic, `file_size` |
| Speed | decisions per forward pass, encoder evaluations per turn, candidates scored per turn, microseconds per decision, milliseconds per training step | counted costs are the deterministic primary metric; `clock_ms()` is the machine-dependent secondary, reported with the binary version |
| Quality | per-question accuracy (each Choice, each Noul, the Scale, recall), top-2, per-category accuracy, NLL, Brier, ECE, reliability bins, coverage at target accuracy, teacher/student KL, abstention rate | the evaluation harness over the fixed split, with the rule engine as oracle |

**Accuracy alone is not a result.** Every quality claim carries the
accuracy-versus-coverage curve, because the number an application actually needs
is "at what threshold can code act autonomously, and how much traffic does that
cover":

| Threshold | Answered automatically | Accuracy when answered |
|---|---|---|
| 0.50 | ... | ... |
| 0.60 | ... | ... |
| 0.70 | ... | ... |
| 0.80 | ... | ... |
| 0.90 | ... | ... |

Every run appends one row to `docs/reference/results.md`, keyed by lesson ID,
configuration, and binary version.

## Experimental progression

Two tracks. The **primitive track** is the subject; the **demo track** is the
forcing function that keeps the primitives honest.

| Stage | Lesson | Track | Mechanism | Question it answers |
|---|---|---|---|---|
| 0 | `EZ01` | demo | classic ELIZA engine in MLPL (keywords, ranks, decomposition, reassembly) | the symbolic baseline and the labeling oracle |
| 0a | `EZ02` | demo | the response table and deterministic transforms, isolated | what is the bounded output space, exactly? |
| 0b | `MB01` | demo | keyword-matcher yardstick on the fixed split | what must a learned model beat to have earned its place? |
| 1 | `DS01` | demo | corpus generator, nine curriculum categories | can we make labels cheaply and know what they teach? |
| 2 | `PR01` | primitive | **Choice**: hashed features -> pool -> one head | can a tiny model answer a typed choice at all? |
| 3 | `PR02` | primitive | **Noul**: independent sigmoid propositions, composed BCE | many independent yes/no beliefs from one state |
| 4 | `PR03` | primitive | **Scale**: ordered levels, ordered loss, expectation | is an ordered scale better than a 3-way classifier? |
| 5 | `PR04` | primitive | all three in **one forward pass** from a shared encoder | the parallel-decisions claim, measured against N separate models |
| 6 | `PR05` | primitive | **dynamic choice sets**: `f(h_state, h_question, h_choice_i)` | can choices be *inputs*, so unseen candidates can be scored? |
| 7 | `CB01` | primitive | reliability, ECE, Brier, NLL; temperature / Platt / isotonic | does `.80` actually mean 80%? |
| 8 | `CB02` | primitive | calibration fine-tuning (`L_CE + lambda L_Brier`) | does training for calibration beat fixing it afterwards? |
| 9 | `AB01` | primitive | `OTHER` / `NONE` training; coverage curves | what does abstention buy, and at what threshold? |
| 10 | `MM01` | primitive | structured memory store and deterministic retrieval | state outside the model |
| 11 | `MM02` | primitive | the `RECALL` decision; Decide -> Gather -> Decide | the model requests an operation; code performs it |
| 12 | `TS01` | primitive | independent scoring -> `grade_down` top-k -> comparative choice | the high-cardinality two-stage path |
| 13 | `TE01` | primitive | local Ollama teachers, first-token logprob distributions | honest soft labels without trusting stated confidence |
| 14 | `TE02` | primitive | three-teacher ensemble; disagreement as a signal | does consensus improve corpus quality? |
| 15 | `KD01` | primitive | soft-label distillation, `KL(teacher \|\| student)` vs `CE` | is the teacher's uncertainty worth keeping? |
| 16 | `AD01` | primitive | active distillation: student uncertainty selects what to label | how far does teacher cost fall? |
| 17 | `EN01` | primitive | encoder ladder: BoW -> pool -> MLP -> CNN -> GRU -> attention -> SSM | how much encoder does a decision surface need? |
| 18 | `MX01` | primitive | MoE encoder | is conditional compute worth it at this scale? |
| 19 | `RB01` | primitive | INT8 / INT4, packed file, microseconds per decision | what does the deployed artifact cost? |
| 20 | `EZ03` | demo | the full demo: chat UI, trace toggle, comparison mode | does the whole chain hold together for a user? |
| 21 | `NV01` | demo | **demo 02**: campus navigation, `lib/` unchanged | are the primitives real abstractions or ELIZA-shaped? |

`MB01` lands early and deliberately: `../moe-microscope` recorded a docent that
did not beat a deterministic matcher, and that honesty is the standard here.
Every learned lesson reports its margin over the matcher on the same split, and
a negative margin is published as a result.

## Saga 1: foundation, probes, and the decision contract

1. **foundation-contract.** `agentrail setup`; `AGENTS.md`, `CLAUDE.md`,
   `justfile`, `mlplunit.conf`, `scripts/`, `catalog/lessons.toml`, the docs
   skeleton, `COPYRIGHT`, `LICENSE`, and the README.
2. **language-probes.** The three Saga-1 probes above, each with a pinned
   mlplunit probe and a findings entry.
3. **decision-contract.** `lib/decision.mlpl`: the `Decision` record, the
   `choice` / `noul` / `scale` signatures, and the `confidence` / `margin` /
   `expectation` definitions — fixed once, used by everything after. Tests pin
   the record shape so later sagas cannot drift it.
4. **trace-schema.** `schemas/decision-trace-v1.schema.json`, the `tdm-trace`
   crate, and a hand-written example trace that validates. The schema exists
   before the first model, so no lesson invents its own format.

Exit: the contract and the trace format are fixed and tested, and the probes
have answers.

## Saga 2: the forcing function — ELIZA, its oracle, and the corpus

1. **eliza-engine (EZ01).** The classic engine in MLPL: keyword list with ranks,
   decomposition patterns, reassembly rules, the pronoun transform, and the
   memory stack ELIZA itself had. Tests pin its output on a fixed transcript so
   it is a *deterministic oracle*, not an approximation of one.
2. **response-table (EZ02).** The response table as committed data, with the
   byte-identity gate test and a diagram of the bounded output space.
3. **matcher-yardstick (MB01).** The keyword matcher scored on the evaluation
   split, with the held-out-lexicon split defined here so no later lesson can
   quietly choose an easier one.
4. **corpus-generator (DS01).** Template and mutation generator producing all
   nine curriculum categories, with the oracle labeling everything it can. The
   split is fixed here and never regenerated.

Exit: a deterministic ELIZA runs, its output space is enumerated and gated, the
yardstick has a published number, and the corpus exists with its split frozen.

## Saga 3: the three primitives

1. **choice (PR01).** Hashed word+bigram features (porting the technique from
   `../moe-microscope/lib/docent.mlpl`), mean pool, one Choice head,
   `cross_entropy`, `adam`. Parameter count printed in full. First pinned trace;
   first instrument render.
2. **noul (PR02).** Eight independent propositions with composed binary
   cross-entropy, and the diagram of eight sigmoids over one shared state.
3. **scale (PR03).** Ordered levels with an ordered loss, the expectation, and
   the ablation that justifies it: ordered loss versus plain 3-way
   cross-entropy, scored on adjacent-versus-distant error rate.
4. **one-pass (PR04).** All three primitives from one shared encoder in a single
   forward pass, measured against the same decisions computed by separate
   models: accuracy, parameters, and encoder evaluations per turn. This is the
   "many questions in parallel against one state" claim, tested rather than
   repeated.
5. **primitives-vs-matcher.** PR04 against MB01 on the same split, per category.

Exit: all three primitives implemented and measured, the shared-encoder saving
is a number, and the margin over the matcher is published per category.

## Saga 4: dynamic choice sets

1. **question-conditioned scorer (PR05).** `score_i = f(h_state, h_question,
   h_choice_i)`, trained over `(state, question, choices, answer)` tuples so the
   same model answers questions whose choices it never saw.
2. **generality-cost.** PR05 against the fixed-head PR01/PR04 on the questions
   both can answer. Generality is not free; the price is a measurement, not an
   assumption.
3. **cross-encoder-comparison.** The accurate-but-repeated cross encoder against
   the shared-state encoder, on accuracy and on encoder evaluations per turn.

Exit: the general `(state, question, choices) -> distribution` form works, and
its cost against fixed heads is published.

## Saga 5: calibration and abstention

1. **calibration-measurement (CB01).** Reliability bins, ECE, Brier, NLL, and
   the reliability diagram composed from `scatter` + `line` + `baseline`.
2. **post-hoc-calibration.** Temperature, Platt, and isotonic (PAVA) from
   scratch, fitted on validation, compared on one table: accuracy, ECE, Brier,
   before and after.
3. **calibration-training (CB02).** `L = L_CE + lambda L_Brier`, swept over
   lambda, against the post-hoc baselines. The honest question this saga exists
   to answer: *at this scale, is anything beyond temperature scaling necessary?*
   A "no" is a publishable result.
4. **abstention (AB01).** `OTHER` / `NONE` with deliberate positives and hard
   negatives; the coverage-versus-accuracy table; the policy idiom
   `if confidence >= t { act } else { fallback }` written in MLPL.
5. **instrument-calibration.** The trace and the instrument show whether a
   distribution is raw or calibrated, and by what map.

Exit: the reliability diagram, the three-way calibration comparison, and the
coverage table are in the results file and render in the instrument.

## Saga 6: memory and two-stage selection

1. **memory-store (MM01).** The structured memory record, deterministic
   extraction, and the cheap retriever (keyword overlap, topic tag, recency)
   with its *own* accuracy number — retrieval quality measured separately from
   decision quality, because conflating them is how this gets misread.
2. **recall-decision (MM02).** The full Decide -> Gather -> Decide -> Act loop,
   with `NONE` in every candidate set and the `memory-negative` category
   carrying real weight.
3. **two-stage-selector (TS01).** Independent sigmoid scoring of up to 100
   memories, `grade_down` top-k, then a comparative softmax over the survivors;
   compared against single-stage selection over the same candidates on accuracy
   and on candidates scored per turn. This is the mechanism TypeSafe describes
   for high-cardinality choices; we arrived at it independently for this problem,
   and the write-up says both things plainly.
4. **memory-instrument.** The instrument's memory pane: the store, the retrieved
   candidates, the second decision, and the `NONE` threshold.

Exit: "Earlier you said..." works without a context window, and the cost of
conversation length is measured rather than assumed.

## Saga 7: local teachers and distillation

Teachers run **offline, locally, through Ollama**. Inference speed does not
matter much — the corpus is generated once and the teachers are then thrown
away — but keeping it fast keeps the loop iterable, so the roster is small
models from three different families, for genuine disagreement rather than three
views of one tokenizer:

| Role | Model | Why |
|---|---|---|
| Teacher A | `llama3.2:3b` | recent small Llama; strong instruction following at 3B |
| Teacher B | `qwen3:4b` | different family, different tokenizer and pretraining mix |
| Teacher C | `gemma3:4b` | a third family, so consensus means something |
| Spare | `llama3.2:1b` | speed floor; how weak a teacher still helps |

All four are in the Ollama library; the local install is `0.32.9`. The step
pulls what is missing (`ollama pull llama3.2:3b`, `qwen3:4b`, `gemma3:4b`) and
records each model's digest in the fixture so a corpus can always be traced back
to the exact weights that produced it. Larger models already on this machine
(`devstral-small-2:24b`, `gemma4:31b-mlx`) are available as an opt-in
high-authority tier for the hard set only, where the cost is affordable because
the set is small.

**Teachers make decisions, not sentences.** The teacher is never asked to write
an ELIZA response — only to answer our typed question over our choice set. This
is the distinction that keeps the corpus on-target.

**Honest soft labels, verified mechanism.** Ollama `0.32.9` returns real
per-token log probabilities (`"logprobs": true, "top_logprobs": 5`), confirmed on
this machine. So the teacher is prompted to answer with a single label token
chosen so that every legal answer has a *distinct first token* (`A`, `B`,
`C`, …), and the distribution is read from the first generated token's
`top_logprobs`, renormalized over the legal set. That yields a genuine
distribution from the teacher's own head — not a number the model was asked to
state about itself, which is precisely the overconfidence TypeSafe attributes to
conventional LLMs and which we must not import into our corpus.

1. **teacher-harness (TE01).** The Ollama client, the single-token label
   protocol, the renormalization, digest recording, caching (identical prompts
   are never re-run), and the fixture schema. Includes the sanity check that the
   legal tokens actually appear in `top_logprobs`, and a documented skip status
   when Ollama is unavailable so the gate still passes on a machine without it.
2. **teacher-calibration.** Before any distillation: measure each teacher's own
   ECE and Brier against the rule oracle on the `easy` category, where truth is
   exact. A teacher whose `.9` means `.6` gets its distribution used, but never
   its confidence trusted — and now we can say by how much.
3. **teacher-ensemble (TE02).** Three teachers, agreement as a training weight,
   disagreement routed to the hard set. Corpus quality is measured as *downstream
   student accuracy*, never as a self-reported score.
4. **distillation (KD01).** `KL(teacher || student)` against plain `CE`, and the
   mixed objective, with the label-authority hierarchy enforced in code so the
   rule oracle always outranks a teacher.
5. **active-distillation (AD01).** Cheap student triage: confident examples skip
   the teacher, uncertain ones are labeled. Report teacher calls saved at equal
   student accuracy — the number that makes this worth doing.

Exit: the distillation pipeline runs end to end offline on local models, each
teacher's calibration is published, and teacher cost per point of student
accuracy is a curve.

## Saga 8: the encoder frontier and the deployed artifact

1. **encoder-ladder (EN01).** Bag-of-words logistic regression, mean pool, MLP,
   CNN, GRU, 1-head attention, and a state-space block — same corpus, same
   heads, same split. Plotted as accuracy *and* ECE against parameters and
   microseconds per decision, so the knee of the curve is visible.
2. **moe-encoder (MX01).** A routed encoder, connecting to
   `../moe-microscope`'s conditional-compute work.
3. **resource-budget (RB01).** INT8 then INT4 weights, the packed file, bytes
   and microseconds per decision, and the quality lost at each step.
4. **pocket-engine.** The deployed artifact: the packed file plus the demo's own
   data, running the whole loop with no generative model present.

Exit: the smallest configuration meeting a stated accuracy-and-calibration bar is
identified by measurement, and it ships as a file.

## Saga 9: the demo UI

1. **chat-first (EZ03).** The conversation UI: plain ELIZA, playback of a pinned
   trace, no instrument visible. It has to be pleasant to use with tracing off,
   or the thesis is not demonstrated.
2. **trace-toggle.** The instrument as an overlay: stepping, state in, choices
   in, typed output with confidence bars, the policy rule, and the table entry.
   Every decision kind renders, including dynamic choice sets.
3. **comparison-mode.** Classic selector and learned selector side by side over
   the same input against the same response table.
4. **ambiguity-theatre.** The deliberately ambiguous inputs — `I dreamed my
   mother was a computer.` — with the policy panel editable, so a viewer watches
   `if confidence < .50 -> DEFAULT` and `if dream > .30 -> DREAM` produce
   different behavior from *identical* model output. Prediction and policy,
   visibly separated.
5. **live-path.** The `mlpl-serve` SSE path: type a turn, run the model, stream
   a fresh trace into the same view model.

Exit: the page is usable as a chat, the trace toggle explains everything behind
it, and both playback and live paths work.

## Saga 10: demo 02, and the findings

1. **navigation-demo (NV01).** Campus navigation through the same primitives:
   `choice` over destinations, `noul` over intent propositions, `scale` over
   urgency, memory over the visit history. **`lib/` may not change.** Whatever
   forces a change to `lib/` is the real finding of this saga, and it is written
   up as one.
2. **instrument-reuse.** The navigation trace renders in the same instrument
   with no new Rust.
3. **findings.** What a tiny typed decision model can and cannot do; where
   calibration training paid and where temperature scaling was enough; what the
   learned decisions bought over keyword matching, per category, with the
   held-out-lexicon result stated plainly; each teacher's calibration and the
   teacher-cost curve; the parameter / latency / accuracy / ECE frontier; and an
   explicit list of what we could *not* determine about Jev from public
   information, so the boundary between our measurements and their claims stays
   legible.

Exit: two demos share one library and one instrument, and the report says what
was learned and what was not.

## Cross-cutting gates

- Every executable behavior starts with native mlplunit coverage; `just check`
  is the full pre-commit gate and grows with each lesson's demo and fixture
  freshness checks.
- Every `.mlpl` file has a module-purpose comment, every user function has a
  first-expression docstring, and canonical formatting is checked before commit
  and push with `scripts/check-mlpl-style`.
- **The no-generation gate.** A test asserts that every string the demo can emit
  is exactly one of the candidates the program offered the model — the selected
  candidate, or a committed frame with deterministically retrieved text spliced
  into it. It runs on every commit. It is the repository's central claim, so it
  is enforced mechanically rather than trusted.
- **The abstraction gate.** No ELIZA identifier may appear in `lib/` or in
  `crates/`. Checked by a script, not by review.
- Every lesson satisfies the visual and measurement contract and adds a catalog
  entry naming its source, tests, previews, trace, fixture, triple, and
  implementation form.
- Every quality claim carries its per-category breakdown, its calibration
  numbers, and its margin over the `MB01` matcher on the same split.
- Every teacher-derived fixture records the model name, digest, prompt revision,
  and sampling settings that produced it.
- Every language or host gap gets a reproducer under `probes/`, a pinned
  mlplunit probe, a workaround, and a findings entry, in the same step it is met.
- Lessons are deterministic, bounded, standalone, and honest about which work
  runs in MLPL, in a builtin, in Rust, or in a teacher.
- Sibling repositories remain read-only; their work is written as a handoff.
- `sw-checklist` applies to the Rust crates.

## Non-goals

- A better chatbot. Response quality in demo 01 is fixed by a 1966 table and is
  not an objective; only *decision* quality is measured.
- Any generative path at inference. No decoder, no sampling into text, no string
  the program did not construct before the model was asked.
- Reproducing Jev, its architecture, or RLCD. We reproduce the published
  *behavioral contract* and the *objective*, and say so every time.
- Claiming schema conformance implies correctness. A typed output cannot fall
  outside the permitted space; it can still be the wrong choice, which is what
  the coverage tables are for.
- Treating any teacher's stated confidence as a calibrated probability.
- ELIZA-specific types in `lib/` or in the Rust crates.
- Teacher inference as a research contribution; Ollama is a tool here, and the
  teachers are discarded once the corpus exists.
- Training on a microcontroller; the MCU / NPU / FPGA track is an inference
  target and is out of scope until after Saga 8.
- Modifying any sibling repository from a saga in this repository.
