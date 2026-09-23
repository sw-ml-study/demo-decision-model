# Writing something that survives a hostile reader

Every page here is a claim about a program. A reader who wants to believe it has
to be able to check it; a reader who wants to break it should find the weak spot
already named, with a number beside it. These are the rules that get us there.
Each one exists because something in this repository was once written without
it.

## 1. Name what you are not counting

A line count, a parameter count, a "runs in three seconds" — each is measured
against a boundary, and the boundary is the interesting part. The concise hello
claims 38 lines and then spends a section on the 284 lines of library those 38
can reach, and on what the program it is compared against stands on. If a number
has a boundary, draw it in the same page.

**Enforcement:** the reachability count comes from the same call-graph walk
`scripts/bundle-program` uses, not from a hand count.

## 2. Print the evidence against yourself

The strongest thing a demo can do is show its own failure, in its own output,
every run. `concise-hello.mlpl` prints how many features it recognized in each
input, so the reader sees that a wire-fraud request was called legitimate at
0.999 confidence off the single word "the". That line is pinned in the gate: if
it ever stops appearing, the document is wrong and the build fails.

A weakness you disclose is a weakness a reader can stop hunting for. One you
omit is the first thing they find.

## 3. Every number in prose is produced by a program the gate runs

No figure is typed from memory or copied from an old run.

- `scripts/check-typed-decisions` runs both demo programs and fails if the lines
  their documents quote change.
- `scripts/check-browser-program` re-bundles the in-browser trainer, runs it
  with no source directory, and requires its measured accuracy.
- `scripts/check-tangle` re-runs every primer block in every literate document
  and compares with the recorded `#+RESULTS`, and re-tangles every source block
  to prove the prose describes code that exists.
- `demos/eliza/sim/tests/rules.rs` holds model 4's agreement with its oracle at
  the value the write-up cites.

## 4. Put the sample size next to the number, and say when it is too small

"0.097 expected calibration error" means little without "on 24 held-out
messages", and the document that reports it also says, in the same paragraph,
that 24 messages cannot pin an ECE to the third decimal. Saying so costs
nothing and removes the reader's best objection.

## 5. Quote a baseline

An accuracy with nothing to compare it against is decoration. `TD01` reports
0.750 against a 0.375 majority class; `RC01` reports 84.9% against a 30.1%
majority class; `SL01` reports its margin over the `MB01` keyword matcher on the
same split. The repository's measurement contract requires this for every
quality claim, and `docs/reference/results.md` is where the row lives.

## 6. State what it cannot do, in the same document

Not in a footnote, not in an issue. The concise hello says outright that a
pretrained model carries knowledge about the world while this one carries
knowledge about twelve sentences, and that this asymmetry cuts against the page.
`DC01` leads with the fact that its scorer picks the right card 0.4% of the time
when that card was held out of training.

## 7. Name the version, the command, and the seed

`mlpl-repl 0.22.0`; `just concise-hello`; `randn(3, ...)`. A result nobody can
re-run is an anecdote.

## 8. Link the artifact, not just its description

Source, tangled program, published page, and the run. A reader should be at most
one click from the thing itself.

## 9. Do not claim a win the numbers do not support

"Thirty-eight against twenty-five is not a win and is not meant to be one." If
the honest reading of your own table is *worse*, write that sentence yourself.
The alternative is that a reader writes it for you, in public, with your table
as the evidence.

## 10. Answer the obvious objection inside the page

Write the section a sceptic would write. The concise hello answers six: isn't
this logistic regression, why not scikit-learn, your confidences are
meaningless, isn't it keyword matching, the comparison is unfair, and how do I
know these numbers are real. Answering them honestly is cheaper than defending
them later, and the answers are usually the most interesting part of the page.

## What this looks like in practice

When a page is close to finished, read it as the reader who wants it to be
wrong, and write down the three things they would say first. If any of them is
unanswerable with a number, that is the next piece of work — not the next
paragraph.
