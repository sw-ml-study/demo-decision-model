# SL01 dialog probe: why the live demo's replies miss

The thin-slice model (`SL01`) scores well on its own splits and poorly in
conversation. This page records how badly, why, and what the evidence says to
do about it. The grading of replies as sensible or not is the author's
judgment, applied by hand; treat the rates as indicative, not measured.

## Method

Two probe sets, run through the same Rust model the live page runs
(`crates/tdm-model`, parity-checked against MLPL):

- [`demos/eliza/dialogs.txt`](../../demos/eliza/dialogs.txt) — 60 things people
  actually type to ELIZA: hedges, questions to the therapist, loss, work, sleep,
  hostility, thanks, goodbye.
- [`demos/eliza/dialogs-wild.txt`](../../demos/eliza/dialogs-wild.txt) — 36
  memes, keyboard noise, and everyday situations.

```sh
cargo run -p tdm-model --example probe        -- fixtures/bundles/demo01.json < demos/eliza/dialogs.txt
cargo run -p tdm-model --example collisions   -- fixtures/bundles/demo01.json < demos/eliza/dialogs.txt
cargo run -p tdm-model --example unknown_gate -- fixtures/bundles/demo01.json < demos/eliza/dialogs.txt
cargo run -p eliza-sim                       -- fixtures/bundles/demo01.json < demos/eliza/conversation.txt
```

## What happens

| Probe set | Nonsensical replies | Features the model trained on | Unseen, borrowed by collision | Unseen, untrained slot |
|---|---|---|---|---|
| realistic (60) | about 45% (27) | 43% | 19% | 38% |
| wild (36) | about 58% (21) | 27% | 27% | 47% |

Of the 27 nonsensical replies in the realistic set, **17 came with confidence
0.85 or higher**, several at 1.00. The model is not unsure and wrong; it is sure
and wrong. `asdf` is DESIRE 1.00. `skill issue` is DREAM 1.00. `goodbye` is
FAMILY 1.00.

## Why

1. **Hash collisions make unseen words impersonate trained ones.** The
   training vocabulary is 459 features in 371 of 1,024 slots. `I don't know`
   reaches the model as `robots_are` (`dont_know` shares its slot) and
   `feel_empty` (`i_dont` shares that one), so it is confidently COMPUTER.
   `goodbye` shares a slot with `mom_never`; `maybe` with `being_chased`. The
   model is not confused: it is reading different words from the ones typed.
2. **Untrained slots add noise to the mean.** 38 to 47% of probe features land
   on slots no training sentence touched. Their vectors are still the seeded
   random initialization, and the mean pool averages them in.
3. **Nine classes, no way to say "none of these".** FALLBACK was trained on
   small talk about topics, not on inputs unlike anything seen. Half the
   failures have no correct label at all: there is no class for "I don't know",
   a question to the therapist, thanks, goodbye, hostility, loss, work, sleep, or
   ELIZA's classic "always" and "everyone".
4. **Frame scaffolding became evidence.** Every COMPUTER frame addresses "you"
   ("are you one of those {}"), so any sentence containing "you" — "how are
   you", "thank you", "I hate you" — leans COMPUTER.
5. **Trained to saturation.** Loss reached about 1e-6 on 232 sentences with
   33,065 parameters and no regularization, which pushes nearly every output to
   1.00 and leaves the 0.4 threshold with nothing to do.
6. **The reply ignores the input.** Even with the right class, the reply is
   picked by turn number, so "my girlfriend cheated on me" gets "What was it
   like growing up with them?".

## Two experiments, in the policy only

**Ignore unseen features and abstain without trained content**
(`unknown_gate`): 36 of 60 decisions change, 34 to abstention. It removes the
confident nonsense, and it also discards about seven replies that were right
("are you a computer?", "my wife wants a divorce"), because their key words
were never in training. Safe, and bland.

**Give abstention something to say** (`demos/eliza/sim`): recall an earlier "my ..."
statement, as Weizenbaum's MEMORY rule did when no keyword matched, else reflect
the input with pronouns swapped. On a 15-turn conversation, "I don't know" goes
from "Do you think machines understand people?" to "Does that have anything to
do with the fact that your mom never listens to you?", and "my dog died" from
"Tell me more about your family." to "Why do you say your dog died?". Every
reply is still a fixed frame plus the user's own words, so nothing is
generated. Visible remaining faults: "nobody cares about me" still reaches DREAM
through the trained word `about`, and the recall sometimes fires when reflecting
the current input would be better.

## The correction that matters: the model's own confidence

Both experiments fix what the page *says*. Neither fixes what the model
*reports*, and that is the repository's subject. Post-hoc temperature scaling
on the validation split will not help: validation is in-distribution, the model
is rightly confident there, and a fitted temperature stays near 1. Uncertainty
on "I don't know" has to be learned from inputs unlike the training classes.

## Recommended order

1. **Stop the impersonation.** Replace hashing with an exact vocabulary plus a
   zero UNK vector (or enough slots that collisions are rare), export the
   vocabulary with the bundle, and treat unknown words as no evidence rather
   than as someone else's evidence.
2. **Teach "none of these".** An explicit OTHER class trained on off-topic,
   noise, and meme inputs, and word dropout to UNK during training, so that
   sparse evidence yields a flat distribution. Add early stopping and label
   smoothing. Then measure calibration (`CB01`) on an out-of-distribution set,
   not only on validation.
3. **Cover the conversation people actually have.** Add DONT_KNOW, QUESTION (to
   the therapist), THANKS, BYE, HOSTILE, SELF_CRITICISM (always, everyone,
   nobody), and LOSS, WORK, HEALTH topics; grow the corpus by paraphrase from
   the local teachers (`TE01`) so it is not all frames one person wrote.
4. **Reflection and memory as first-class behaviour**, not a fallback hack:
   the decision chooses the move (reflect, recall, class reply), a second
   decision chooses which memory (`MM02`), and deterministic code composes.
5. **Choose the reply, not just the class**: offer the class's candidate replies
   as a composed choice set instead of rotating by turn number.

## Training time is a knob, not a goal

The target is a usable demo, not fast training (user direction, 2026-09-20).
`SL01` trains in 24 s only because nothing asked it to train longer. Step `008`
therefore compares the same fixed model and data under two wall-clock budgets
— **10 seconds and 5 minutes** — each in its own process (finding Q4: an
in-process sweep inherits Adam's moments and flatters whichever run went last).
Each budget is scored on the same four numbers: the sensible-reply rate on both
probe sets, and the mean confidence the model reports on the answers it gets
wrong, in-distribution and out. A **1 hour** run is deferred until those two
results say it is worth running (user direction, 2026-09-20). The demo ships the
cheapest budget that reaches the quality bar.

These probe sets become the regression benchmark for each step: the sensible
rate and, more importantly, the confidence the model reports when it is wrong.
