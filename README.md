# demo-decision-model

Tiny models that decide, not talk.

**Live demo: <https://sw-ml-study.github.io/demo-decision-model/>** — nothing to
clone or install. The trained model runs entirely in your browser (WASM, about
250 KB gzipped including the weights), and a trace view shows every decision
behind every reply. A link can carry the conversation, e.g.
[`?say=my+mom+never+listens+to+me`](https://sw-ml-study.github.io/demo-decision-model/?say=my+mom+never+listens+to+me).

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

The model can choose wrongly. It cannot say anything the program did not already
construct — and a gate test asserts that on every commit. Sometimes the options
are assembled in full by ordinary code (`Tell me more.`, `Earlier you said you
bought a new car.`) and the model simply ranks them; sometimes code splices a
stored memory into a fixed frame. Either way the model ranks bounded choices and
composes nothing.

That is a sharper demonstration of the "no hallucination" claim than any
statement about JSON schemas: schema conformance does not mean the decision is
correct, it means the output cannot fall outside the permitted space.

Demo 02 (campus navigation) must run with `lib/` unchanged. That is the test of
whether the primitives are real abstractions or ELIZA-shaped ones.

## Status

**Running.** There is a demo you can talk to, and a model that decides what it
says.

```sh
just eliza-train                              # about 25 s, writes the weights
just chat "my mom never listens to me"
just transcript                               # the committed nine-turn demo
```

```text
YOU:   my mom never listens to me
ELIZA: Tell me more about your family.

      choice: what should the reply be? 9 offered
      FAMILY #################### 1
      DREAM  .................... 0
      YES    .................... 0
      DESIRE .................... 0
      confidence 1  margin 1  decided in 1.041 ms
      policy: confidence >= 0.4, act on FAMILY
      reply:  table FAMILY entry 0 (verbatim, not composed)
      matcher: FALLBACK (1966 keyword list)
      features: my|mom|never|listens|to|me|my_mom|mom_never|never_listens|...
```

That is the whole thesis in one screen: text in, a distribution over nine
bounded options, a threshold applied by ordinary code, and a reply quoted from a
table. The 1966 keyword list falls through on `mom`, because nobody wrote `mom`
down in 1966. The model learned it from examples.

### What it measures

A single Choice over nine response classes. 33,065 parameters
(1,024 hash slots x 32 dimensions, plus a 32x9 head), deciding in **0.5 to
2.1 ms**; the nine-turn transcript runs end to end in 0.11 s.

**It starts from nothing.** The embedding table and head are seeded random
values (`randn(...) * 0.1`), the bias is zero, and the only thing the model ever
sees is the 232 generated sentences. No pretrained vectors, no downloaded
weights, no teacher yet. That is also why `mom` works and `the woman who raised
me` does not: `mom` was in the corpus, and a hash slot for a word the model never
saw carries no meaning at all.

**Training is a one-off.** 200 full-batch Adam steps over 232 examples,
**23.9 s** wall clock on an M1 Max (`mlpl-repl 0.22.0`), writing a 264,688-byte
weights file. It is deterministic: retraining reproduces the file byte for byte
(verified by hash). The weights are committed, so a fresh clone can
`just chat` immediately and never run training at all. Retraining is only needed
when the classes, the corpus, or the hyperparameters change.

| Split | Learned Choice | 1966 keyword matcher | Margin |
|---|---|---|---|
| train (232) | 1.000 | 0.526 | +0.474 |
| val (58, held-out sentence frames) | 0.879 | 0.397 | +0.483 |
| wild (20, hand-written) | 0.750 | 0.400 | +0.350 |

### What it gets wrong, which matters more

The model is **badly calibrated and confidently wrong off-distribution**:

| Input | Model says | Truth | Matcher |
|---|---|---|---|
| `yeah` | `YES` 1.000 | `YES` | falls through |
| `do you think robots have feelings` | `COMPUTER` 0.995 | `COMPUTER` | falls through |
| `i bought a new car` | `DESIRE` 0.786 | `FALLBACK` | `FALLBACK`, correct |
| `i dreamed my mother was a computer` | `DESIRE` 0.883 | `DREAM` | `DREAM`, correct |
| `the woman who raised me never approved` | `DREAM` 0.705 | `FAMILY` | falls through |

It reports `1.000` on most inputs. Those are not probabilities yet, which is
precisely why calibration (`CB01`, `CB02`) and abstention (`AB01`) are their own
sagas rather than a footnote. Validation `COMPUTER` scores 0.000: the held-out
frame for that class is `i spend all day with {}`, and nothing in training
prepared it.

**This is a thin slice, marked `provisional`.** One Choice, no Noul, no Scale, no
memory, no calibration, no browser UI. It exists so there is something real to
judge before anything is deepened. Numbers: [`SL01`](docs/reference/results.md).

### Several questions per input: the Noul heads

Model 3 ([`NH01`](catalog/lessons.toml)) asks three yes-or-no questions of every
input -- *is it a question? is it negative? is it positive?* -- from the same
forward pass as the reply Choice: one pooled state, four typed decisions. A
question mark is visible to it now, where the cleaner used to drop it.

Weizenbaum's ELIZA did not answer questions; it turned them back. So a question
is deflected first -- "We were discussing you, not me.", "I ask the questions
here.", "I'm not here to answer questions. What do you think?" -- and never by
echoing it. When the model cannot decide, a longer input is reflected with its
sentiment and a short one brings back a memory.

On the 96 frozen probes the question Noul catches 12 of 13 questions with a 1%
false-alarm rate; the negative Noul catches 66% of negative inputs; the positive
Noul catches only 1 of 5, because positives are rare in both the training data
and the probes. The default snapshot is chosen by validation accuracy over all
four heads, not the Choice alone: the Choice peaks at 2.3 minutes while the
Nouls keep improving to 12, and choosing by the Choice would have shipped a
question detector that missed half the questions. Model 3 escalates 19% of
inputs, against a third for model 2, with the same accuracy when it replies.

### ELIZA's mechanics, back in the program

The model only decides what kind of reply fits. What made ELIZA feel like ELIZA
was the program around that decision, and it now lives here too, driven by a
data script ([`demo01-script.json`](fixtures/bundles/demo01-script.json)) that a
demo-neutral engine in `tdm-model` runs:

- **Memory, kept by the program.** Every statement is remembered with its
  content words.
- **"Earlier you said..."** when a visitor reuses a content word from an earlier
  turn, and Weizenbaum's MEMORY rule (bring back a remembered "my ..." statement)
  when nothing else fits. Each memory is brought back once, and never twice in a
  row.
- **Decomposition and reassembly** keyed by the model's choice: `* i want *`
  becomes "What would it mean to you if you got a better job?", with the
  visitor's words reflected (my -> your). "I don't have any problems" gets
  Weizenbaum's own "Don't you really have any problems?".

Every reply is a fixed frame with slots filled by the visitor's own reflected
words, the trace shows the frame and the slots, and a test rebuilds every reply
from exactly those parts. Nothing is generated.

### Model 2: fewer confident wrong replies

The first live model often asked about machines, or greeted, when the visitor
had done neither. A probe traced both to hash collisions (unseen words reading as
trained ones) and to cues in the training frames (*you* alone pushed toward
COMPUTER). Model 2 ([`EH01`](catalog/lessons.toml)) replaces hashing with an exact
vocabulary, adds a trained "none of these" class and an explicit escalation
outcome, and covers seventeen kinds of reply. Scored on 96 hand-labelled probes
whose labels were frozen before any new training frame was written:

| What a visitor sees | Model 1 | Model 2 |
|---|---|---|
| reply class correct | 40% | **69%** |
| right when it does not escalate | 38% | **78%** |
| wrong "machines" replies | 12 | **1** |
| wrong greetings | 7 | **0** |
| escalated to "I can't tell" | 9% | 33% |

On the 78 probes never seen in training it is 62% against 38%, and it is 0.36
confident when wrong against 0.72. A third of inputs still escalate, and in the
browser that means a flat "Please go on."; reflection, recall, and the ELIZA-
oracle corpus are next. Details: [`SL01-dialog-probe.md`](docs/experiments/SL01-dialog-probe.md).

### Training, visible

The page shows training, not only inference. One MLPL run from random weights
is snapshotted after 0, 1, 2, 5 and 10 seconds of training; pick a snapshot and
the whole conversation is re-decided by the model as it was then
([`TL01`](catalog/lessons.toml)):

| Snapshot | Val accuracy | Wild accuracy | Confidence on probe inputs |
|---|---|---|---|
| 0 s, random | 0.069 | 0.10 | 0.115 |
| 1 s | 0.845 | **0.80** | 0.514 |
| 2 s | **0.879** | 0.75 | 0.740 |
| 5 s | 0.879 | 0.75 | 0.808 |
| 10 s | 0.879 | 0.75 | 0.811 |

At 0 s every class scores about 0.11 and the policy abstains on everything. The
learning happens in the first second. After two seconds accuracy stops moving
and the only thing training still buys is confidence, including on the 95
probe inputs the model was never trained on. For a model whose job is to report
how sure it is, that is the problem this repository exists to fix. A link can
open on a snapshot: [`?snap=0`](https://sw-ml-study.github.io/demo-decision-model/?snap=0&say=i+feel+sad).

### In the browser

`crates/tdm-model` ports the featurizer and forward pass to Rust; MLPL remains
the only trainer. `demos/eliza/export.mlpl` writes the trained model as a JSON
bundle that carries 319 parity inputs with MLPL's own probabilities and matcher
picks, and the Rust tests require all of them to match: largest disagreement
under `1e-9`, the matcher identical on every input. Rounding the weights to
`1e-6` for transport changed no decision.

`crates/tdm-web` is the Yew page. It embeds the bundle, so the site is one
static download: 2.43 MB of WASM, 865 KB gzipped, most of it the five
embedded training snapshots. In memory the model is 33,065 f64 values, 264 KB. It is built locally
into `pages/` with `just site`, committed, and published unchanged by a GitHub
Actions workflow; CI builds nothing. The footer names the source commit it was
built from, the host, and the time, and marks a build from uncommitted source
as `-dirty`.

### Foundations under it

- [`docs/plan.md`](docs/plan.md) — the delivery plan and the authority for scope.
- [`lib/decision.mlpl`](lib/decision.mlpl) — the `Decision` record and the three
  primitives, pinned by 13 tests.
- [`schemas/decision-trace-v1.schema.json`](schemas/decision-trace-v1.schema.json)
  and [`crates/tdm-trace`](crates/tdm-trace) — the trace format and validator.
  It rejects a fourth decision kind, and rejects any output text that was not
  one of the candidates the program offered.
- [`probes/`](probes/) — four sw-MLPL capability reproducers. The fourth cost a
  wrong number before it was caught: Adam's optimizer state is process-global, so
  an in-process sweep reports whatever ran last as the winner. Every sweep here
  runs one configuration per process.

Next: the real `EZ01` rule engine as a labelling oracle, a corpus with proper
curriculum categories, then Noul and Scale beside the Choice.

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

Directories arrive as their saga does; the demos and the instrument crates are
still to come.

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
