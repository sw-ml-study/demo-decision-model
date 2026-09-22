# RC01: a Choice over another program's rules, learned from that program's decisions

Model 3's labels were seventeen classes one person invented, scored against a
keyword list the same person wrote. Model 4 drops all of that. Its Choice is
over **the rules of Weizenbaum's DOCTOR script**, and its labels come from
running that script over
[8,464 inputs nobody here labelled](OR01-oracle-corpus.md).

The question this answers is not "can a small model imitate ELIZA". It is
whether the three primitives still hold up when the label set, the training
data and the yardstick all come from outside: **Choice** over 35 rules,
**Noul** for three typed yes-or-no questions on the same forward pass, and
**Memory** kept by the program, not the model.

## What changed

| | model 3 (NH01) | model 4 (RC01) |
|---|---|---|
| Labels | 17 classes, written here | 35 rules of the 1966 script |
| Training inputs | 1,000 generated sentences | 7,595 oracle-labelled inputs |
| Who wrote the labels | this repository | the DOCTOR script |
| Yardstick | a keyword matcher written here | the script itself |
| Reply | a frame from this repository's script | the chosen rule's own reassembly |
| Vocabulary | 1,291 exact tokens | 2,000 exact tokens |
| Parameters | 41,905 | 65,286 |

### The classes are rules, and rare rules are merged

The oracle used 45 rules. A rule with fewer than eight training examples cannot
be learned from and is merged into its keyword's most common rule, or into
`xnone#0` when that keyword has none: `i#1` into `i#11`, `can#1` into `can#0`,
`was#0` and `was#2` into `was#1`, and `am#0`, `am#1`, `why#0`, `why#1`,
`remember#0`, `remember#1` into the catch-all. 35 classes remain. This is a real
loss: `i#1` is the script's "I am sorry to hear you are depressed", and the
model can no longer choose it.

### Two labels the data already carried

The Noul heads need labels the corpus does not state. Two came free:

- **question**: a sentence pulled out of a novel ends in `?` or it does not.
  The author punctuated it; nobody here did. 1,001 of the 7,000 dialogue
  sentences are questions on that evidence.
- **negative** and **positive**: only the generated sentences carry sentiment
  tags, so those two columns are **masked** everywhere else.
  `u:cm_masked_joint_loss` averages the binary cross-entropy over labelled
  cells only, so a novel sentence trains the question head and leaves the other
  two alone.

### Minibatches, because full batch stopped being affordable

One full-batch step over 7,595 rows costs about **30 s** in the interpreter: a
ten-minute budget would buy twenty steps. Adam's moments are process-global and
persist between calls, so calling the trainer once per minibatch of 256 is one
continuous optimization: **1,268 steps, 42 epochs, in the same ten minutes**.

### The featurizer moved to Rust, the learning did not

Building eight thousand rows of feature lists is quadratic in the interpreter.
`demos/eliza/sim/src/bin/featurize.rs` writes the id and mask matrices, the
class list and the vocabulary; `demos/eliza/v4/export.mlpl` does every bit of
the learning. The parity test still holds the two featurizers to 1e-9 on the
weights MLPL produced.

One bug worth recording: the vocabulary is the 2,000 most frequent training
tokens, and **"hello" lost an alphabetical tie at the cutoff** (count 6). The
most common opening line in the demo was an unknown word, so the model answered
it from the bias alone at 0.06 confidence. The vocabulary now always includes
every word the script itself names — keywords, synonym members, pattern
literals — before filling up by frequency.

## Results

Training snapshots, each one scored identically; the default is chosen by mean
validation accuracy over all four heads, never by the probes.

| Budget | Steps | Epochs | Val agreement | Probe agreement | Question Noul acc | Val all heads |
|---|---:|---:|---:|---:|---:|---:|
| 0 s | 0 | 0 | 0.040 | 0.021 | 0.583 | 0.368 |
| 10 s | 20 | 0.6 | 0.352 | 0.365 | 0.885 | 0.669 |
| 1 min | 122 | 4.1 | 0.764 | 0.844 | 0.990 | 0.896 |
| **5 min** | **606** | **20.4** | **0.849** | **0.938** | **0.990** | **0.950** |
| 10 min | 1,268 | 42.7 | 0.825 | 0.948 | 0.990 | 0.944 |

The trend the earlier budgets predicted holds again: most of the accuracy
arrives early, and past five minutes validation agreement **falls** while
confidence keeps rising. An hour would buy nothing this model can use.

At the default snapshot (5 min):

| | validation (869) | probes (96) |
|---|---:|---:|
| agreement with the oracle | 84.9% | 93.8% |
| majority-class baseline | 30.1% | 21.9% |
| the chosen rule's pattern fits the input | 97.1% | 97.9% |
| reply identical to the oracle's, word for word | 85.5% | 93.8% |
| policy: act / none applies / escalate | 58.2% / 32.2% / 9.6% | 72.9% / 19.8% / 7.3% |
| mean confidence when right / wrong | 0.72 / 0.43 | 0.80 / 0.49 |
| expected calibration error (10 bins) | 0.169 | 0.159 |

The Nouls, at the same snapshot: question 0.99 accurate on the probes (12 of 13
questions caught, no false alarms) and 0.992 on the validation split; negative
0.82 on the probes, 0.992 on the labelled validation rows; positive still weak
where it always was — 0.92 accurate but catching only 1 of 5, because positive
examples are rare in every source we have.

The full per-rule breakdown is what `just model4-score` prints; the two
agreement numbers are pinned as a ratchet in
`demos/eliza/sim/tests/rules.rs`.

### What the numbers do not say

- **93.8% on the probes is the easier number.** The probes are short, typical
  openings; the validation split is novel dialogue, which is longer and
  stranger. 84.9% is the honest one.
- **Agreement is not quality.** The oracle is itself a 1966 keyword program,
  and a third of its decisions are "I'm not sure I understand you fully". The
  model is being judged on reproducing that, not on being a good listener.
- **The model is underconfident, not overconfident**, which is the direction we
  want: it is right 85% of the time while claiming 0.72, and when it is wrong
  it claims 0.43. The escape hatch and the escalation threshold catch the rest.

## What the page does with it

The model picks a rule; the **script** answers with that rule's own
reassembly, filled with the visitor's own words. Nothing is generated: the
reply is a frame from the 1966 script and text the program already had, and the
no-generation test rebuilds every reply from exactly those parts. When the
chosen rule's pattern does not fit the words (2.9% of the time), the rule's
reassembly that needs no captured part answers instead.

The rest of the conversation is unchanged and still lives outside the model:
the repeat rule ("Earlier you said …"), the MEMORY rule, deflection of
questions, and escalation when the model cannot tell.
