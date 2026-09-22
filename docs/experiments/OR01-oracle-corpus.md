# OR01: a labelling oracle and a corpus nobody here wrote the labels for

Every earlier model learned labels the author assigned to frames the author
wrote, scored against a keyword list the author wrote. This step replaces that
with Weizenbaum's own decisions.

## The oracle

[`demo01-doctor.json`](../../fixtures/bundles/demo01-doctor.json) is the 1966
DOCTOR script as data: 35 keywords with ranks, 52 decomposition rules, synonym
groups (`@family`, `@sad`, `@desire`, ...), pre- and post-substitution, `goto`
between keywords, and the MEMORY rule. It is reconstructed after the widely
circulated version by Charles Hayden. A demo-neutral engine in
`crates/tdm-model/src/keyword.rs` runs it, and reports for every input **which
rule fired** (`key#i`) and what it said.

It behaves as the published transcript does, and tests pin it:

| Input | Rule | Reply |
|---|---|---|
| Men are all alike. | `alike#0` | In what way? |
| Well, my boyfriend made me come here. | `my#2` | Your boyfriend made you come here? |
| He says I'm depressed much of the time. | `i#1` | I am sorry to hear that you are depressed. |
| I don't have any problems | `i#8` | Don't you really have any problems? |
| because we did | `because#0` | Is that the real reason? |

Two bugs in the first draft of the script were caught by working out the
reassembly numbering before trusting it (a stray wildcard in a `@belief`
pattern, and a pattern on `cant` that pre-substitution to `cannot` made
unreachable).

## The corpus

8,464 distinct inputs, none equal to one of the 96 frozen probes, split
by a stable hash into 7,595 training and 869
validation. A sentence that ended in a question mark keeps it: the author's own
punctuation is a free label for "is this a question?", which is what model 4's
question Noul learns from.

| Source | Inputs | What it is |
|---|---|---|
| novel dialogue | 7,000 | every quoted sentence of 3 to 18 words from ten public-domain novels on Project Gutenberg (Pride and Prejudice, Alice, Sherlock Holmes, Little Women, Great Expectations, Emma, Dorian Gray, Anne of Green Gables, Dracula, Frankenstein), capped at 700 per book |
| everyday lines | 180 | slang, memes, workplace jargon and ordinary situations, written for this corpus |
| generated frames | 1,284 | model 3's template sentences, relabelled by the oracle |

Movie quotes were left out: the repository is public and they are not.

## What ELIZA does with ordinary speech

The oracle used 45 of its rules. The distribution is its own finding:
**31% of inputs match no keyword at all**
(`xnone`, "I'm not sure I understand you fully."), and the next most common is
`i#11`, the generic "You say ...?". ELIZA was mostly deflecting.

| Rule | Inputs | Share |
|---|---|---|
| `xnone#0` | 2,635 | 31.1% |
| `i#11` | 1,064 | 12.6% |
| `my#2` | 775 | 9.2% |
| `what#0` | 755 | 8.9% |
| `you#3` | 753 | 8.9% |
| `if#0` | 282 | 3.3% |
| `i#6` | 226 | 2.7% |
| `i#10` | 190 | 2.2% |
| `your#0` | 162 | 1.9% |
| `no#0` | 162 | 1.9% |
| `you#1` | 123 | 1.5% |
| `always#0` | 120 | 1.4% |
| ... 33 more | 1,217 | |

## Found on the way

The novels spell apostrophes curly, and so do iPhones by default. Both
featurizers deleted only the straight one, so "I don’t know" became "i don t
know" and missed every `dont` rule -- on the live demo, for anyone typing on a
phone. Both cleaners now treat `’` and `‘` as apostrophes, with a test.

## Next

Train model 4 to choose the oracle's rule, measure its agreement with the oracle
on validation and on the probes, and let ELIZA's own reassembly write the reply.
