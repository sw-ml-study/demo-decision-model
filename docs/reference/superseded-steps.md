# Planned steps that later work answered

Two steps queued early in the saga were satisfied by work that took a different
route. Closing them without a record would make the saga disagree with the
repository; this page is that record.

## `eliza-engine` — the keyword engine, and where it ended up

**Asked for:** the classic keyword / rank / decomposition / reassembly engine in
MLPL, as a deterministic oracle, with its output pinned on a fixed transcript.

**What answered it:** [`OR01`](../experiments/OR01-oracle-corpus.md) built that
engine, in Rust, because the job it was wanted for turned out to be labelling
8,464 inputs — which the MLPL interpreter builds lists for quadratically.

- `crates/tdm-model/src/keyword.rs` is the engine: ranked keywords, `@synonym`
  groups, pre- and post-substitution, `goto` between keywords, the MEMORY rule,
  and reassembly with the input's own words spliced in.
- `fixtures/bundles/demo01-doctor.json` is the 1966 script as committed data:
  35 keywords, 52 decomposition rules.
- `demos/eliza/sim/tests/oracle.rs` pins it on exchanges from the published
  transcript — five tests, including the four lines everyone quotes.

**What was given up by not writing it in MLPL:** nothing MLPL needs today. Every
lesson that consumes the oracle consumes its *output* (the corpus in
`demos/eliza/oracle/corpus.tsv`), and MLPL reads that as data. A second engine
in MLPL would be a second thing to keep in step with the first, and the moment
they disagreed the labels would quietly change.

**When it should be reopened:** if a lesson ever has to label data *inside* an
MLPL program — generating negatives during training, say, rather than reading
them from a file. Then the engine has to exist where the training loop is.

## `response-table` — the table as data, and the no-generation gate

**Asked for:** the ELIZA response table as committed data, plus the
byte-identity no-generation gate test.

**What answered it:** both halves exist, in the form the demos actually took.

- The tables are committed data: `fixtures/bundles/demo01-script.json` (the
  conversation script: frames, recall, memory, deflection) and
  `demo01-doctor.json` (the 1966 script). Model 1's nine-class table lives in
  `demos/eliza/eliza.mlpl`, where it is generated rather than written out.
- The gate is enforced in both languages. In MLPL,
  `tests/test_eliza.mlpl` walks every class and every turn the response
  selector can reach and requires the text to be **byte-identical** to an entry
  of that class's table. In Rust, four tests rebuild every reply of a session
  from its frame and slots and require the result to equal what was said —
  which is the form the claim takes once a frame carries the visitor's own
  words.

**The wording changed, and that was the point.** "Byte-identical to a table
entry" was true of model 1 and false the moment a reply could quote the visitor
("Earlier you said *your mom never listens to you*."). The invariant this
repository enforces is the stronger, more honest one:
[`plan.md`](../plan.md)'s *every string a user sees is exactly one of the
candidates the program offered the model* — a committed frame with
deterministically retrieved text spliced in, rebuildable from exactly those
parts. A test that still demanded byte-identity would have had to be deleted or
weakened; instead the claim was restated so it survives contact with memory.
